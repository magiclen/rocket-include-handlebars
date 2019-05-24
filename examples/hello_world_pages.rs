#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_handlebars;

#[macro_use]
extern crate handlebars;

extern crate json_gettext;

use std::collections::HashMap;

use rocket::State;
use rocket_include_handlebars::{EtagIfNoneMatch, HandlebarsResponse, HandlebarsContextManager};

use json_gettext::JSONGetTextValue;

#[get("/")]
fn index() -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!("index", &map)
}

#[get("/etag")]
fn index_etag(etag_if_none_match: EtagIfNoneMatch) -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!(etag_if_none_match, "index", &map)
}

#[get("/2")]
fn index_2(cm: State<HandlebarsContextManager>) -> HandlebarsResponse {
    handlebars_response_static!(
        cm,
        "index2",
        {
            println!("Generate index_2 and staticize it...");

            let mut map = HashMap::new();

            map.insert("title", JSONGetTextValue::from_str("Title"));
            map.insert("placeholder", JSONGetTextValue::from_str("Hello, \"world!\""));
            map.insert("id", JSONGetTextValue::from_u64(0));

            handlebars_response!("index2", &map)
        }
    )
}

#[get("/2/etag")]
fn index_2_etag(etag_if_none_match: EtagIfNoneMatch, cm: State<HandlebarsContextManager>) -> HandlebarsResponse {
    handlebars_response_static!(
        etag_if_none_match,
        cm,
        "index2etag",
        {
            println!("Generate index_2_etag and staticize it...");

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
        .attach(HandlebarsResponse::fairing(|handlebars| {
            handlebars_resources_initialize!(
                handlebars,
                "index", "examples/views/index.hbs",
                "index2", "examples/views/index2.hbs"
            );

            handlebars_helper!(inc: |x: i64| x + 1);

            handlebars.register_helper("inc", Box::new(inc));
        }))
        .mount("/", routes![index, index_etag])
        .mount("/", routes![index_2, index_2_etag])
        .launch();
}