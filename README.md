Include Handlebars Templates for Rocket Framework
====================

[![Build Status](https://travis-ci.org/magiclen/rocket-include-handlebars.svg?branch=master)](https://travis-ci.org/magiclen/rocket-include-handlebars)

This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.

* `handlebars_resources_initialize!` is used in the fairing of `HandlebarsResponse` to include Handlebars files into your executable binary file. You need to specify each file's name and its path. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `handlebars_response!` is used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally not be minified.
* `handlebars_response_cache!` is used for wrapping a `HandlebarsResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.

See `examples`.

## Crates.io

https://crates.io/crates/rocket-include-handlebars

## Documentation

https://docs.rs/rocket-include-handlebars

## License

[MIT](LICENSE)