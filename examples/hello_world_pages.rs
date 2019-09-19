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
use rocket_include_handlebars::{HandlebarsContextManager, HandlebarsResponse};

use json_gettext::JSONGetTextValue;

#[get("/")]
fn index() -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!("index", &map)
}

#[get("/disable-minify")]
fn index_disable_minify() -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!(disable_minify "index", &map)
}

#[get("/2")]
fn index_2(cm: State<HandlebarsContextManager>) -> HandlebarsResponse {
    handlebars_response_cache!(cm, "index-2", {
        println!("Generate index-2 and cache it...");

        let mut map = HashMap::new();

        map.insert("title", JSONGetTextValue::from_str("Title"));
        map.insert("placeholder", JSONGetTextValue::from_str("Hello, \"world!\""));
        map.insert("id", JSONGetTextValue::from_u64(0));

        handlebars_response!(auto_minify "index2", &map)
    })
}

fn main() {
    rocket::ignite()
        .attach(HandlebarsResponse::fairing(|handlebars| {
            handlebars_resources_initialize!(
                handlebars,
                "index",
                "examples/views/index.hbs",
                "index2",
                "examples/views/index2.hbs"
            );

            handlebars_helper!(inc: |x: i64| x + 1);

            handlebars.register_helper("inc", Box::new(inc));

            // NOTE: The above `inc` helper can be alternately added by enabling the `helper_inc` feature for the rocket_include_handlebars crate.
        }))
        .mount("/", routes![index, index_disable_minify])
        .mount("/", routes![index_2])
        .launch();
}
