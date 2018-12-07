Include Handlebars Templates for Rocket Framework
====================

[![Build Status](https://travis-ci.org/magiclen/rocket-include-handlebars.svg?branch=master)](https://travis-ci.org/magiclen/rocket-include-handlebars)

This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.

## Example

```rust
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate lazy_static_include;

#[macro_use] extern crate handlebars;

#[macro_use] extern crate rocket_include_handlebars;

#[macro_use] extern crate rocket;

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

## Crates.io

https://crates.io/crates/rocket-include-handlebars

## Documentation

https://docs.rs/rocket-include-handlebars

## License

[MIT](LICENSE)