Include Handlebars Templates for Rocket Framework
====================

[![CI](https://github.com/magiclen/rocket-include-handlebars/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/rocket-include-handlebars/actions/workflows/ci.yml)

This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.

## Example

```rust
#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_handlebars;

use std::collections::HashMap;

use rocket::State;

use rocket_include_handlebars::{EtagIfNoneMatch, HandlebarsContextManager, HandlebarsResponse};

#[get("/")]
fn index(
    handlebars_cm: &State<HandlebarsContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!(handlebars_cm, etag_if_none_match, "index", map)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(handlebars_resources_initializer!(
            "index" => "views/index.hbs",
        ))
        .mount("/", routes![index])
}
```

* `handlebars_resources_initialize!` is used in the fairing of `HandlebarsResponse` to include Handlebars files into your executable binary file. You need to specify each file's name and its path relative to the directory containing the manifest of your package. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `handlebars_response!` renders a registered template as a `HandlebarsResponse`.
  A `200 OK` response includes `Content-Type`, `Content-Length`, and `ETag`.
  A `304 Not Modified` response includes the `ETag` and has no body.
  You can turn off HTML minification.
* `handlebars_response_cache!` is used for wrapping a `HandlebarsResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
* `handlebars_resources_initializer!` is used for generating a fairing for handlebars resources.

## JSON Object Key Order

`serde_json` sorts the keys of JSON objects in alphabetical order by default, so `{{#each}}` visits the keys of an object in that order.
Enable the `preserve_json_order` feature to keep the insertion order instead.

```toml
[dependencies]
rocket-include-handlebars = { version = "0.14", features = ["preserve_json_order"] }
```

This feature turns on the `preserve_order` feature of `serde_json`, which affects every crate in your dependency tree.

## Crates.io

https://crates.io/crates/rocket-include-handlebars

## Documentation

https://docs.rs/rocket-include-handlebars

## License

[MIT](LICENSE)
