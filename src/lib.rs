/*!
# Include Handlebars Templates for Rocket Framework

This is a crate which provides macros `handlebars_resources_initialize!` and `handlebars_response!` to statically include HBS (Handlebars) files from your Rust project and make them be the HTTP response sources quickly.

* `handlebars_resources_initialize!` is used in the fairing of `HandlebarsResponse` to include Handlebars files into your executable binary file. You need to specify each file's name and its path relative to the directory containing the manifest of your package. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `handlebars_response!` is used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally not be minified.
* `handlebars_response_cache!` is used for wrapping a `HandlebarsResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
* `handlebars_resources_initializer!` is used for generating a fairing for handlebars resources.

See `examples`.
*/

#[macro_use]
extern crate educe;

#[doc(hidden)]
pub extern crate manifest_dir_macros;

mod functions;

#[cfg(debug_assertions)]
mod debug;

#[cfg(not(debug_assertions))]
mod release;

mod macros;

#[cfg(debug_assertions)]
pub use debug::*;

#[cfg(not(debug_assertions))]
pub use release::*;

pub use handlebars::handlebars_helper;

pub use rocket_etag_if_none_match::entity_tag::EntityTag;
pub use rocket_etag_if_none_match::EtagIfNoneMatch;

const DEFAULT_CACHE_CAPACITY: usize = 64;
