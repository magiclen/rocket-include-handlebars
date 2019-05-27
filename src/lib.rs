/*!
# Include Handlebars Templates for Rocket Framework

This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.

* `handlebars_resources_initialize!` is used in the fairing of `HandlebarsResponse` to include Handlebars files into your executable binary file. You need to specify each file's name and its path. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `handlebars_response!` is used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally not be minified.
* `handlebars_response_cache!` is used for wrapping a `HandlebarsResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.

See `examples`.
*/

mod reloadable;
mod manager;
mod fairing;
mod macros;

pub extern crate handlebars;

#[macro_use]
extern crate derivative;
extern crate crc_any;
extern crate html_minifier;
extern crate rc_u8_reader;
extern crate lru_time_cache;

extern crate serde;

extern crate serde_json;

extern crate rocket;

extern crate rocket_etag_if_none_match;

use std::io::Cursor;
#[cfg(debug_assertions)]
use std::sync::MutexGuard;
use std::sync::Arc;

use crc_any::CRC;
use rc_u8_reader::ArcU8Reader;
use handlebars::{Handlebars, RenderError};
use serde::Serialize;
use serde_json::{Value, Error as SerdeJsonError};

use rocket::State;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::Status;
use rocket::fairing::Fairing;

pub use rocket_etag_if_none_match::{EntityTag, EtagIfNoneMatch};

pub use reloadable::ReloadableHandlebars;
pub use manager::HandlebarsContextManager;
use fairing::HandlebarsResponseFairing;

const DEFAULT_CACHE_CAPACITY: usize = 64;

#[inline]
fn compute_html_etag<S: AsRef<str>>(html: S) -> EntityTag {
    let mut crc64ecma = CRC::crc64ecma();
    crc64ecma.digest(html.as_ref().as_bytes());
    let crc64 = crc64ecma.get_crc();
    EntityTag::new(true, format!("{:X}", crc64))
}

#[derive(Debug)]
enum HandlebarsResponseSource {
    Template {
        minify: bool,
        name: &'static str,
        context: Value,
    },
    Cache(Arc<str>),
}

#[derive(Debug)]
/// To respond HTML from Handlebars templates.
pub struct HandlebarsResponse {
    client_etag: EtagIfNoneMatch,
    source: HandlebarsResponseSource,
}

impl HandlebarsResponse {
    #[inline]
    /// Build a `HandlebarsResponse` instance from a specific template.
    pub fn build_from_template<V: Serialize>(client_etag: EtagIfNoneMatch, minify: bool, name: &'static str, context: V) -> Result<HandlebarsResponse, SerdeJsonError> {
        let context = serde_json::to_value(context)?;

        let source = HandlebarsResponseSource::Template {
            minify,
            name,
            context,
        };

        Ok(HandlebarsResponse {
            client_etag,
            source,
        })
    }

    #[inline]
    /// Build a `HandlebarsResponse` instance from cache.
    pub fn build_from_cache<S: Into<Arc<str>>>(client_etag: EtagIfNoneMatch, name: S) -> HandlebarsResponse {
        let source = HandlebarsResponseSource::Cache(name.into());

        HandlebarsResponse {
            client_etag,
            source,
        }
    }
}

impl HandlebarsResponse {
    #[cfg(debug_assertions)]
    #[inline]
    /// Create the fairing of `HandlebarsResponse`.
    pub fn fairing<F>(f: F) -> impl Fairing where F: Fn(&mut MutexGuard<ReloadableHandlebars>) + Send + Sync + 'static {
        let f = Box::new(f);

        HandlebarsResponseFairing {
            custom_callback: Box::new(move |handlebars| {
                f(handlebars);

                DEFAULT_CACHE_CAPACITY
            }),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Create the fairing of `HandlebarsResponse`.
    pub fn fairing<F>(f: F) -> impl Fairing where F: Fn(&mut Handlebars) + Send + Sync + 'static {
        let f = Box::new(f);

        HandlebarsResponseFairing {
            custom_callback: Box::new(move |handlebars| {
                f(handlebars);

                DEFAULT_CACHE_CAPACITY
            }),
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    /// Create the fairing of `HandlebarsResponse` and set the cache capacity.
    pub fn fairing_cache<F>(f: F) -> impl Fairing where F: Fn(&mut MutexGuard<ReloadableHandlebars>) -> usize + Send + Sync + 'static {
        HandlebarsResponseFairing {
            custom_callback: Box::new(f),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Create the fairing of `HandlebarsResponse` and set the cache capacity.
    pub fn fairing_cache<F>(f: F) -> impl Fairing where F: Fn(&mut Handlebars) -> usize + Send + Sync + 'static {
        HandlebarsResponseFairing {
            custom_callback: Box::new(f),
        }
    }
}

impl HandlebarsResponse {
    #[cfg(debug_assertions)]
    #[inline]
    fn render(&self, cm: &HandlebarsContextManager) -> Result<String, RenderError> {
        match &self.source {
            HandlebarsResponseSource::Template {
                name,
                context,
                ..
            } => {
                cm.handlebars.lock().unwrap().render(name, context)
            }
            _ => unreachable!()
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    fn render(&self, cm: &HandlebarsContextManager) -> Result<String, RenderError> {
        match &self.source {
            HandlebarsResponseSource::Template {
                name,
                context,
                ..
            } => {
                cm.handlebars.render(name, context)
            }
            _ => unreachable!()
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    /// Get this response's HTML and Etag.
    pub fn get_html_and_etag(&self, cm: &HandlebarsContextManager) -> Result<(Arc<str>, Arc<EntityTag>), RenderError> {
        match &self.source {
            HandlebarsResponseSource::Template {
                name,
                context,
                ..
            } => {
                let html = cm.handlebars.lock().unwrap().render(name, context)?;

                let etag = compute_html_etag(&html);

                Ok((html.into(), Arc::new(etag)))
            }
            HandlebarsResponseSource::Cache(key) => {
                cm.get(key).ok_or(RenderError::new("This Response hasn't triggered yet."))
            }
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Get this response's HTML and Etag.
    pub fn get_html_and_etag(&self, cm: &HandlebarsContextManager) -> Result<(Arc<str>, Arc<EntityTag>), RenderError> {
        match &self.source {
            HandlebarsResponseSource::Template {
                name,
                context,
                ..
            } => {
                let html = cm.handlebars.render(name, context)?;

                let etag = compute_html_etag(&html);

                Ok((html.into(), Arc::new(etag)))
            }
            HandlebarsResponseSource::Cache(key) => {
                cm.get(key).ok_or(RenderError::new("This Response hasn't triggered yet."))
            }
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    /// Get this response's HTML.
    pub fn get_html(&self, cm: &HandlebarsContextManager) -> Result<String, RenderError> {
        match &self.source {
            HandlebarsResponseSource::Template {
                name,
                context,
                ..
            } => {
                let html = cm.handlebars.lock().unwrap().render(name, context)?;

                Ok(html)
            }
            HandlebarsResponseSource::Cache(key) => {
                cm.get(key).map(|(html, _)| html.to_string()).ok_or(RenderError::new("This Response hasn't triggered yet."))
            }
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Get this response's HTML.
    pub fn get_html(&self, cm: &HandlebarsContextManager) -> Result<String, RenderError> {
        match &self.source {
            HandlebarsResponseSource::Template {
                name,
                context,
                ..
            } => {
                let html = cm.handlebars.render(name, context)?;

                Ok(html)
            }
            HandlebarsResponseSource::Cache(key) => {
                cm.get(key).map(|(html, _)| html.to_string()).ok_or(RenderError::new("This Response hasn't triggered yet."))
            }
        }
    }
}

impl<'a> Responder<'a> for HandlebarsResponse {
    fn respond_to(self, request: &Request) -> response::Result<'a> {
        let mut response = Response::build();

        let cm = request.guard::<State<HandlebarsContextManager>>().expect("HandlebarsContextManager registered in on_attach");

        match &self.source {
            HandlebarsResponseSource::Template {
                minify,
                ..
            } => {
                let (html, etag) = match self.render(&cm) {
                    Ok(html) => {
                        let etag = compute_html_etag(&html);

                        let is_etag_match = self.client_etag.weak_eq(&etag);

                        if is_etag_match {
                            response.status(Status::NotModified);

                            return response.ok();
                        } else {
                            (html, etag.to_string())
                        }
                    }
                    Err(_) => {
                        response.status(Status::InternalServerError);

                        return response.ok();
                    }
                };

                let html = if *minify {
                    html_minifier::minify(&html).unwrap()
                } else {
                    html
                };

                response
                    .raw_header("ETag", etag)
                    .raw_header("Content-Type", "text/html; charset=utf-8")
                    .sized_body(Cursor::new(html));
            }
            HandlebarsResponseSource::Cache(key) => {
                let (html, etag) = {
                    match cm.get(key) {
                        Some((html, etag)) => {
                            let is_etag_match = self.client_etag.weak_eq(&etag);

                            if is_etag_match {
                                response.status(Status::NotModified);

                                return response.ok();
                            } else {
                                (html, etag.to_string())
                            }
                        }
                        None => {
                            response.status(Status::InternalServerError);

                            return response.ok();
                        }
                    }
                };

                response
                    .raw_header("ETag", etag)
                    .raw_header("Content-Type", "text/html; charset=utf-8")
                    .sized_body(ArcU8Reader::new(html));
            }
        }

        response.ok()
    }
}