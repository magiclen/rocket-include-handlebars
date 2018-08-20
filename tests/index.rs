#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket_include_handlebars;
extern crate rocket_etag_if_none_match;

extern crate rocket;
#[macro_use]
extern crate handlebars;

handlebars_resources_initialize!(
    "index", "included-handlebars/index.hbs",
    "index-2", "included-handlebars/index2.hbs"
);

use std::collections::HashMap;

use rocket::local::Client;
use rocket::http::Status;

use rocket_include_handlebars::HandlebarsResponse;
use rocket_etag_if_none_match::EtagIfNoneMatch;

#[get("/")]
fn index(etag_if_none_match: EtagIfNoneMatch) -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    handlebars_response!(etag_if_none_match, "index", &map)
}

#[get("/2")]
fn index_2() -> HandlebarsResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("placeholder", "Hello, \"world!\"");
    map.insert("id", "0");

    handlebars_response!("index-2", &map)
}

#[test]
fn test_index() {
    let rocket = rocket::ignite();

    let rocket = rocket
        .mount("/", routes![index]);

    let client = Client::new(rocket).expect("valid rocket instance");

    let req = client.get("/");

    let mut response = req.dispatch();

    assert_eq!(response.body_string(), Some("<!DOCTYPE html><html><head><title>Title</title></head><body>Hello, world!</body>".to_string()));
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap().to_string(), "text/html");
}

#[test]
fn test_index_2() {
    let rocket = rocket::ignite();

    let rocket = rocket
        .mount("/", routes![index_2]);

    let client = Client::new(rocket).expect("valid rocket instance");

    let req = client.get("/2");

    let mut response = req.dispatch();

    assert_eq!(response.body_string(), Some("<!DOCTYPE html><html><head><title>Title</title></head><body><input id=\"input-1\" type=\"text\" placeholder=\"Hello, &quot;world!&quot;\"></body>".to_string()));
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap().to_string(), "text/html");
}