//! # Include Handlebars Templates for Rocket Framework
//! This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.
//!
//! ## Example
//!
//! ```
//! #![feature(plugin)]
//! #![plugin(rocket_codegen)]
//!
//! #[macro_use] extern crate lazy_static;
//! #[macro_use] extern crate lazy_static_include;
//!
//! #[macro_use] extern crate rocket_include_handlebars;
//! extern crate rocket_etag_if_none_match;
//!
//! extern crate rocket;
//! #[macro_use] extern crate handlebars;
//!
//! handlebars_resources_initialize!(
//!     "index", "included-handlebars/index.hbs",
//!     "index-2", "included-handlebars/index2.hbs"
//! );
//!
//! use std::collections::HashMap;
//!
//! use rocket::local::Client;
//! use rocket::http::Status;
//!
//! use rocket_include_handlebars::HandlebarsResponse;
//! use rocket_etag_if_none_match::EtagIfNoneMatch;
//!
//! #[get("/")]
//! fn index() -> HandlebarsResponse {
//!     let mut map = HashMap::new();
//!
//!     map.insert("title", "Title");
//!     map.insert("body", "Hello, world!");
//!
//!     handlebars_response!("index", &map)
//! }
//!
//! #[get("/2")]
//! fn index_2() -> HandlebarsResponse {
//!     let mut map = HashMap::new();
//!
//!     map.insert("title", "Title");
//!     map.insert("body", "Hello, world!");
//!
//!     handlebars_response!("index-2", &map)
//! }
//!
//! #[get("/static")]
//! fn index_static() -> HandlebarsResponse {
//!     handlebars_response_static!(
//!         "index".to_string(),
//!         {
//!             let mut map = HashMap::new();
//!
//!             map.insert("title", "Title");
//!             map.insert("body", "Hello, world!");
//!
//!             handlebars_response!("index", &map)
//!         }
//!     )
//! }
//! ```
//!
//! * `handlebars_resources_initialize!` is used for including HBS files into your executable binary file. You need to specify each file's ID and its path. For instance, the above example uses **index** to represent the file **included-handlebars/index.hbs** and **index-2** to represent the file **included-handlebars/index2.hbs**. An ID cannot be repeating.
//! * `handlebars_response!` is used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
//! * `handlebars_response_static!` is used for in-memory staticizing a `HandlebarsResponse` instance by a given key.
//!
//! Refer to `tests/index.rs` to see the example completely.
//!
//! In order to reduce the compilation time, files are compiled into your executable binary file together, only when you are using the **release** profile.

extern crate rocket;

pub extern crate crc_any;
pub extern crate rocket_etag_if_none_match;
pub extern crate html_minifier;
pub extern crate handlebars;
pub extern crate json_gettext;

use crc_any::CRC;

use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::{Status, hyper::header::{ETag, EntityTag}};

use std::io::Cursor;
use rocket_etag_if_none_match::EtagIfNoneMatch;

pub struct HandlebarsResponse {
    /// The HTML code.
    pub html: String,
    /// The Etag from HTTP client.
    pub etag: EtagIfNoneMatch,
    /// The Etag computed from the HTML code.
    pub my_etag: Option<String>,
    /// If you don't want minify the rendered HTML, set `minify` to **false**.
    pub minify: bool,
}

impl<'a> Responder<'a> for HandlebarsResponse {
    fn respond_to(self, _: &Request) -> response::Result<'a> {
        let my_etag = match self.my_etag {
            Some(my_etag) => my_etag,
            None => {
                let mut crc64ecma = CRC::crc64ecma();
                crc64ecma.digest(self.html.as_bytes());
                let crc64 = crc64ecma.get_crc();
                format!("{:X}", crc64)
            }
        };

        let etag = self.etag.etag;

        let mut is_etag_match = false;

        if let Some(etag) = etag {
            if etag.tag().eq(&my_etag) {
                is_etag_match = true;
            }
        }

        let mut response = Response::build();

        if is_etag_match {
            response.status(Status::NotModified);
        } else {
            let html = if self.minify {
                html_minifier::minify(&self.html).unwrap()
            } else {
                self.html
            };

            response
                .header(ETag(EntityTag::new(true, my_etag)))
                .raw_header("Content-Type", "text/html")
                .raw_header("Content-Length", html.len().to_string())
                .chunked_body(Cursor::new(html), HANDLEBARS_RESPONSE_CHUNK_SIZE);
        }

        response.ok()
    }
}

#[doc(hidden)]
pub const HANDLEBARS_RESPONSE_CHUNK_SIZE: u64 = 4096;

/// Used for including HBS files into your executable binary file. You need to specify each file's ID and its path. For instance, the above example uses **index** to represent the file **included-handlebars/index.hbs** and **index-2** to represent the file **included-handlebars/index2.hbs**. An ID cannot be repeating.
#[macro_export]
macro_rules! handlebars_resources_initialize {
    ( $($id:expr, $path:expr), * ) => {
        lazy_static_include_str_vec!(HANDLEBARS_REG_DATA $(, $path)* );

        lazy_static! {
            static ref HANDLEBARS_REG: ::rocket_include_handlebars::handlebars::Handlebars = {
                {
                    use ::rocket_include_handlebars::handlebars::Handlebars;

                    let mut reg = Handlebars::new();

                    let mut p = 0usize;

                    $(
                        {
                            let template = HANDLEBARS_REG_DATA[p];

                            p += 1;

                            reg.register_template_string($id, template).unwrap();
                        }
                    )*

                    handlebars_helper!(inc: |x: i64| x + 1);

                    handlebars_helper!(dec: |x: i64| x - 1);

                    handlebars_helper!(eq_str: |x: str, y: str| x == y);

                    handlebars_helper!(ne_str: |x: str, y: str| x != y);

                    reg.register_helper("inc", Box::new(inc));
                    reg.register_helper("dec", Box::new(dec));
                    reg.register_helper("eq_str", Box::new(eq_str));
                    reg.register_helper("ne_str", Box::new(ne_str));

                    reg
                }
            };

            static ref HANDLEBARS_STATIC: std::sync::Mutex<std::collections::HashMap<String, (String, String)>> = {
                std::sync::Mutex::new(std::collections::HashMap::new())
            };
        }
    };
}


/// Used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
#[macro_export]
macro_rules! handlebars_response {
    ( $id:expr, $data:expr ) => {
        {
            use ::rocket_include_handlebars::HandlebarsResponse;

            let html = HANDLEBARS_REG.render($id, $data).unwrap();

            HandlebarsResponse{
                html,
                etag: ::rocket_include_handlebars::rocket_etag_if_none_match::EtagIfNoneMatch {etag: None},
                my_etag: None,
                minify: true,
            }
        }
    };
    ( $etag_if_none_match:expr, $id:expr, $data:expr ) => {
        {
            use ::rocket_include_handlebars::HandlebarsResponse;

            let html = HANDLEBARS_REG.render($id, $data).unwrap();

            HandlebarsResponse{
                html,
                etag: $etag_if_none_match,
                my_etag: None,
                minify: true,
            }
        }
    };
}

#[macro_export]
macro_rules! handlebars_response_static {
    ( $key:expr, $gen:block ) => {
        {
            if let Some((html, etag)) = HANDLEBARS_STATIC.lock().unwrap().get($key.as_str()) {
                return HandlebarsResponse{
                    html: html.clone(),
                    etag: ::rocket_include_handlebars::rocket_etag_if_none_match::EtagIfNoneMatch {etag: None},
                    my_etag: Some(etag.clone()),
                    minify: false,
                };
            }

            let mut res = $gen;

            if res.minify {
                res.html = ::rocket_include_handlebars::html_minifier::minify(&res.html).unwrap();
                res.minify = false;
            }

            let my_etag = match res.my_etag {
                Some(my_etag) => my_etag,
                None => {
                    let mut crc64ecma = ::rocket_include_handlebars::crc_any::CRC::crc64ecma();
                    crc64ecma.digest(res.html.as_bytes());
                    let crc64 = crc64ecma.get_crc();
                    format!("{:X}", crc64)
                }
            };

            HANDLEBARS_STATIC.lock().unwrap().insert($key, (res.html.clone(), my_etag.clone()));

            res.my_etag = Some(my_etag);

            res
        }
    };
    ( $etag_if_none_match:expr, $key:expr, $gen:block ) => {
        {
            if let Some((html, etag)) = HANDLEBARS_STATIC.lock().unwrap().get($key.as_str()) {
                return HandlebarsResponse{
                    html: html.clone(),
                    etag: $etag_if_none_match,
                    my_etag: Some(etag.clone()),
                    minify: false,
                };
            }

            let mut res = $gen;

            if res.minify {
                res.html = ::rocket_include_handlebars::html_minifier::minify(&res.html).unwrap();
                res.minify = false;
            }

            let my_etag = match res.my_etag {
                Some(my_etag) => my_etag,
                None => {
                    let mut crc64ecma = ::rocket_include_handlebars::crc_any::CRC::crc64ecma();
                    crc64ecma.digest(res.html.as_bytes());
                    let crc64 = crc64ecma.get_crc();
                    format!("{:X}", crc64)
                }
            };

            HANDLEBARS_STATIC.lock().unwrap().insert($key, (res.html.clone(), my_etag.clone()));

            res.my_etag = Some(my_etag);

            res
        }
    };
}