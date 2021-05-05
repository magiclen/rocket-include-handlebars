#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_handlebars;

#[macro_use]
extern crate serde_json;

use std::collections::HashMap;

use rocket::State;

use rocket_include_handlebars::{EtagIfNoneMatch, HandlebarsContextManager, HandlebarsResponse};

#[get("/")]
fn index(
    handlebars_cm: State<HandlebarsContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!(handlebars_cm, etag_if_none_match, "index", map)
}

#[get("/disable-minify")]
fn index_disable_minify(
    handlebars_cm: State<HandlebarsContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!(disable_minify handlebars_cm, etag_if_none_match, "index", map)
}

#[get("/2")]
fn index_2(
    cm: State<HandlebarsContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> HandlebarsResponse {
    handlebars_response_cache!(cm, etag_if_none_match, "index-2", {
        println!("Generate index-2 and cache it...");

        let json = json! ({
            "title": "Title",
            "placeholder": "Hello, \"world!\"",
            "id": 0,
        });

        handlebars_response!(auto_minify cm, EtagIfNoneMatch::default(), "index2", json)
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(HandlebarsResponse::fairing(|handlebars| {
            handlebars_resources_initialize!(
                handlebars,
                "index" => "examples/views/index.hbs",
                "index2" => ("examples", "views", "index2.hbs")
            );

            handlebars_helper!(inc: |x: i64| x + 1);

            handlebars.register_helper("inc", Box::new(inc));

            // NOTE: The above `inc` helper can be alternately added by enabling the `helper_inc` feature for the rocket_include_handlebars crate.
        }))
        .mount("/", routes![index, index_disable_minify])
        .mount("/", routes![index_2])
}
