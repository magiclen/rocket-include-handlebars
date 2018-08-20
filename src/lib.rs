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
//! ```
//!
//! * `handlebars_resources_initialize!` is used for including HBS files into your executable binary file. You need to specify each file's ID and its path. For instance, the above example uses **index** to represent the file **included-handlebars/index.hbs** and **index-2** to represent the file **included-handlebars/index2.hbs**. An ID cannot be repeating.
//! * `handlebars_response!` is used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
//!
//! Refer to `tests/index.rs` to see the example completely.

extern crate crc;
extern crate rocket;
extern crate rocket_etag_if_none_match;
extern crate minify;
extern crate handlebars;

use crc::{crc64, Hasher64};

use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::{Status, hyper::header::{ETag, EntityTag}};

use std::io::Cursor;
use rocket_etag_if_none_match::EtagIfNoneMatch;
use minify::html::minify;

pub struct HandlebarsResponse {
    pub html: String,
    pub etag: EtagIfNoneMatch,
    /// If you don't want minify the rendered HTML, set `minify` to **false**.
    pub minify: bool,
}

impl<'a> Responder<'a> for HandlebarsResponse {
    fn respond_to(self, _: &Request) -> response::Result<'a> {
        let mut digest = crc64::Digest::new(crc64::ECMA);
        digest.write(self.html.as_bytes());

        let crc64 = digest.sum64();
        let my_etag = format!("{:X}", crc64);

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
                minify(&self.html)
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
        lazy_static! {
            static ref HANDLEBARS_REG: self::handlebars::Handlebars = {
                {
                    use self::handlebars::Handlebars;

                    let mut reg = Handlebars::new();

                    $(
                        {
                            let template = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path));

                            reg.register_template_string($id, template).unwrap();
                        }
                    )*

                    handlebars_helper!(inc: |x: i64| x + 1);
                    handlebars_helper!(inc_str: |x: str| x.parse::<i64>().unwrap() + 1i64);

                    reg.register_helper("inc", Box::new(inc));
                    reg.register_helper("inc_str", Box::new(inc_str));

                    reg
                }
            };
        }
    };
}


/// Used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
#[macro_export]
macro_rules! handlebars_response {
    ( $id:expr, $data:expr ) => {
        {
            use self::rocket_include_handlebars::HandlebarsResponse;

            let html = HANDLEBARS_REG.render($id, $data).unwrap();

            HandlebarsResponse{
                html,
                etag: EtagIfNoneMatch {etag: None},
                minify: true,
            }
        }
    };
    ( $etag_if_none_match:expr, $id:expr, $data:expr ) => {
        {
            use self::rocket_include_handlebars::HandlebarsResponse;

            let html = HANDLEBARS_REG.render($id, $data).unwrap();

            HandlebarsResponse{
                html,
                etag: $etag_if_none_match,
                minify: true,
            }
        }
    };
}