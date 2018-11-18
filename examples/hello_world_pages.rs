#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[macro_use]
extern crate handlebars;

#[macro_use]
extern crate rocket_include_handlebars;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate lazy_static_include;

extern crate json_gettext;

use std::collections::HashMap;

use rocket_include_handlebars::{EtagIfNoneMatch, HandlebarsResponse};

use json_gettext::JSONGetTextValue;

handlebars_resources_initialize!(
    "index", "examples/views/index.hbs",
    "index2", "examples/views/index2.hbs"
);

#[get("/")]
fn index(etag_if_none_match: EtagIfNoneMatch) -> HandlebarsResponse {
    println!("Generate index...");

    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!(etag_if_none_match, "index", &map)
}

#[get("/2")]
fn index_2() -> HandlebarsResponse {
    handlebars_response_static!(
        "index2".to_string(),
        {
            println!("Generate index2 and staticize it...");

            let mut map = HashMap::new();

            map.insert("title", JSONGetTextValue::from_str("Title"));
            map.insert("placeholder", JSONGetTextValue::from_str("Hello, \"world!\""));
            map.insert("id", JSONGetTextValue::from_u64(0));

            handlebars_response!("index2", &map)
        }
    )
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/", routes![index_2])
        .launch();
}