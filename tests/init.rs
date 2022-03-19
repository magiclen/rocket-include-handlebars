use rocket_include_handlebars::*;

#[test]
fn fairing_cache() {
    rocket::build().attach(handlebars_resources_initializer!(
        100;
        "index" => "examples/views/index.hbs",
        "index2" => "examples/views/index2.hbs"
    ));
}

#[test]
fn fairing() {
    rocket::build().attach(handlebars_resources_initializer!(
        "index" => "examples/views/index.hbs",
        "index2" => "examples/views/index2.hbs"
    ));
}
