/*!
# Include Handlebars Templates for Rocket Framework

This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.

* `handlebars_resources_initialize!` is used for including HBS files into your executable binary file. You need to specify each file's name and its path. For instance, the above example uses **index** to represent the file **included-handlebars/index.hbs** and **index-2** to represent the file **included-handlebars/index2.hbs**. A name cannot be repeating. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `handlebars_response!` is used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
* `handlebars_response_static!` is used for in-memory staticizing a `HandlebarsResponse` instance by a given key. It is effective only when you are using the **release** profile.

See `examples`.
*/

mod reloadable;
mod manager;
mod fairing;
mod macros;

pub extern crate handlebars;

extern crate crc_any;
extern crate html_minifier;

extern crate serde;

extern crate serde_json;

extern crate rocket;

extern crate rocket_etag_if_none_match;

use std::io::Cursor;
#[cfg(debug_assertions)]
use std::sync::MutexGuard;

use crc_any::CRC;
use handlebars::{Handlebars, RenderError};
use serde::Serialize;
use serde_json::{Value, Error as SerdeJsonError};

use rocket::State;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::{Status, hyper::header::ETag};
use rocket::fairing::Fairing;

pub use rocket_etag_if_none_match::{EntityTag, EtagIfNoneMatch};

pub use reloadable::ReloadableHandlebars;
pub use manager::HandlebarsContextManager;
use fairing::HandlebarsResponseFairing;

#[inline]
fn compute_html_etag(html: &str) -> EntityTag {
    let mut crc64ecma = CRC::crc64ecma();
    crc64ecma.digest(html.as_bytes());
    let crc64 = crc64ecma.get_crc();
    EntityTag::new(true, format!("{:X}", crc64))
}

#[derive(Debug)]
enum HandlebarsResponseSource {
    Template {
        etag: Option<EntityTag>,
        minify: bool,
        name: &'static str,
        context: Value,
    },
    Cache(Option<String>),
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
    pub fn build_from_template<V: Serialize>(client_etag: EtagIfNoneMatch, etag: Option<EntityTag>, minify: bool, name: &'static str, context: V) -> Result<HandlebarsResponse, SerdeJsonError> {
        let context = serde_json::to_value(context)?;

        let source = HandlebarsResponseSource::Template {
            etag,
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
    /// Build a `HandlebarsResponse` instance from static cache.
    pub fn build_from_cache<S: Into<String>>(client_etag: EtagIfNoneMatch, name: S) -> HandlebarsResponse {
        let source = HandlebarsResponseSource::Cache(Some(name.into()));

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
        HandlebarsResponseFairing {
            custom_callback: Box::new(f)
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Create the fairing of `HandlebarsResponse`.
    pub fn fairing<F>(f: F) -> impl Fairing where F: Fn(&mut Handlebars) + Send + Sync + 'static {
        HandlebarsResponseFairing {
            custom_callback: Box::new(f)
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
    pub fn get_html_and_etag(&self, cm: &HandlebarsContextManager) -> Result<(String, EntityTag), RenderError> {
        match &self.source {
            HandlebarsResponseSource::Template {
                name,
                context,
                ..
            } => {
                let html = cm.handlebars.lock().unwrap().render(name, context)?;

                let etag = compute_html_etag(&html);

                Ok((html, etag))
            }
            HandlebarsResponseSource::Cache(name) => {
                let cache_table = cm.cache_table.lock().unwrap();

                match cache_table.get(name.as_ref().unwrap()) {
                    Some((html, etag)) => Ok((html.clone(), etag.clone())),
                    None => Err(RenderError::new("This Response hasn't triggered yet."))
                }
            }
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Get this response's HTML and Etag.
    pub fn get_html_and_etag(&self, cm: &HandlebarsContextManager) -> Result<(String, EntityTag), RenderError> {
        match &self.source {
            HandlebarsResponseSource::Template {
                name,
                context,
                ..
            } => {
                let html = cm.handlebars.render(name, context)?;

                let etag = compute_html_etag(&html);

                Ok((html, etag))
            }
            HandlebarsResponseSource::Cache(name) => {
                let cache_table = cm.cache_table.lock().unwrap();

                match cache_table.get(name.as_ref().unwrap()) {
                    Some((html, etag)) => Ok((html.clone(), etag.clone())),
                    None => Err(RenderError::new("This Response hasn't triggered yet."))
                }
            }
        }
    }
}

impl<'a> Responder<'a> for HandlebarsResponse {
    fn respond_to(mut self, request: &Request) -> response::Result<'a> {
        let mut response = Response::build();

        let cm = request.guard::<State<HandlebarsContextManager>>().expect("HandlebarsContextManager registered in on_attach");

        let (is_template, etag, minify) = {
            match &mut self.source {
                HandlebarsResponseSource::Template {
                    etag,
                    minify,
                    ..
                } => {
                    (true, etag.take(), *minify)
                }
                _ => (false, None, false)
            }
        };

        if is_template {
            let (html, etag) = match etag {
                Some(etag) => {
                    let is_etag_match = self.client_etag.weak_eq(&etag);

                    if is_etag_match {
                        response.status(Status::NotModified);

                        return response.ok();
                    } else {
                        match self.render(&cm) {
                            Ok(html) => (html, etag),
                            Err(_) => {
                                response.status(Status::InternalServerError);

                                return response.ok();
                            }
                        }
                    }
                }
                None => {
                    match self.render(&cm) {
                        Ok(html) => {
                            let etag = compute_html_etag(&html);

                            let is_etag_match = self.client_etag.weak_eq(&etag);

                            if is_etag_match {
                                response.status(Status::NotModified);

                                return response.ok();
                            } else {
                                (html, etag)
                            }
                        }
                        Err(_) => {
                            response.status(Status::InternalServerError);

                            return response.ok();
                        }
                    }
                }
            };

            let html = if minify {
                html_minifier::minify(&html).unwrap()
            } else {
                html
            };

            response.header(ETag(etag));

            response.raw_header("Content-Type", "text/html; charset=utf-8")
                .sized_body(Cursor::new(html));
        } else {
            let name = if let HandlebarsResponseSource::Cache(name) = &mut self.source {
                name.take().unwrap()
            } else {
                unreachable!()
            };

            let cache = {
                let cache_table = cm.cache_table.lock().unwrap();

                match cache_table.get(&name) {
                    Some((html, etag)) => {
                        let is_etag_match = self.client_etag.weak_eq(etag);

                        if is_etag_match {
                            response.status(Status::NotModified);

                            None
                        } else {
                            Some((html.clone(), etag.clone()))
                        }
                    }
                    None => {
                        response.status(Status::InternalServerError);

                        return response.ok();
                    }
                }
            };

            if let Some((html, etag)) = cache {
                response.header(ETag(etag));

                response.raw_header("Content-Type", "text/html; charset=utf-8")
                    .sized_body(Cursor::new(html));
            }
        }

        response.ok()
    }
}