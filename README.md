Include Handlebars Templates for Rocket Framework
====================

[![Build Status](https://travis-ci.org/magiclen/rocket-include-handlebars.svg?branch=master)](https://travis-ci.org/magiclen/rocket-include-handlebars)

This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.

* `handlebars_resources_initialize!` is used for including HBS files into your executable binary file. You need to specify each file's name and its path. For instance, the above example uses **index** to represent the file **included-handlebars/index.hbs** and **index-2** to represent the file **included-handlebars/index2.hbs**. A name cannot be repeating. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `handlebars_response!` is used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
* `handlebars_response_static!` is used for in-memory staticizing a `HandlebarsResponse` instance by a given key. It is effective only when you are using the **release** profile.

See `examples`.

## Crates.io

https://crates.io/crates/rocket-include-handlebars

## Documentation

https://docs.rs/rocket-include-handlebars

## License

[MIT](LICENSE)