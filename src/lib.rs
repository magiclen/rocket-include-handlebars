/*!
# Include Handlebars Templates for Rocket Framework

This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.

## Example

```rust
#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate lazy_static_include;

#[macro_use] extern crate handlebars;

#[macro_use] extern crate rocket_include_handlebars;

extern crate rocket;

use std::collections::HashMap;

use rocket_include_handlebars::{EtagIfNoneMatch, HandlebarsResponse};

handlebars_resources_initialize!(
    "index", "examples/views/index.hbs",
    "index2", "examples/views/index2.hbs"
);

#[get("/")]
fn index() -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!("index", &map)
}

#[get("/2")]
fn index_2() -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!("index2", &map)
}

#[get("/static")]
fn index_static() -> HandlebarsResponse {
    handlebars_response_static!(
        "index".to_string(),
        {
            let mut map = HashMap::new();

            map.insert("title", "Title");
            map.insert("body", "Hello, world!");

            handlebars_response!("index", &map)
        }
    )
}
```

* `handlebars_resources_initialize!` is used for including HBS files into your executable binary file. You need to specify each file's ID and its path. For instance, the above example uses **index** to represent the file **included-handlebars/index.hbs** and **index-2** to represent the file **included-handlebars/index2.hbs**. An ID cannot be repeating.
* `handlebars_response!` is used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
* `handlebars_response_static!` is used for in-memory staticizing a `HandlebarsResponse` instance by a given key.

In order to reduce the compilation time, files are compiled into your executable binary file together, only when you are using the **release** profile.

See `examples`.
*/

#[doc(hidden)]
pub extern crate handlebars;
#[doc(hidden)]
pub extern crate crc_any;
#[doc(hidden)]
pub extern crate html_minifier;

extern crate rocket;
extern crate rocket_etag_if_none_match;

use crc_any::CRC;

use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::{Status, hyper::header::ETag};

use std::io::Cursor;
pub use rocket_etag_if_none_match::{EntityTag, EtagIfNoneMatch};

pub struct HandlebarsResponse {
    /// The HTML code.
    pub html: String,
    /// The Etag from HTTP client (Etag-If-None-Match).
    pub client_etag: EtagIfNoneMatch,
    /// The Etag computed from the HTML code or provided by a specific one.
    pub etag: Option<EntityTag>,
    /// If you don't want minify the rendered HTML, set `minify` to **false**.
    pub minify: bool,
}

impl<'a> Responder<'a> for HandlebarsResponse {
    fn respond_to(self, _: &Request) -> response::Result<'a> {
        let etag = match self.etag {
            Some(etag) => etag,
            None => {
                let mut crc64ecma = CRC::crc64ecma();
                crc64ecma.digest(self.html.as_bytes());
                let crc64 = crc64ecma.get_crc();
                EntityTag::new(true, format!("{:X}", crc64))
            }
        };

        let is_etag_match = self.client_etag.weak_eq(&etag);

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
                .header(ETag(etag))
                .raw_header("Content-Type", "text/html")
                .raw_header("Content-Length", html.len().to_string())
                .sized_body(Cursor::new(html));
        }

        response.ok()
    }
}

mod macros;