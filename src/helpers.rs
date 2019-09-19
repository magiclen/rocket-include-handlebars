#[cfg(feature = "helper")]
macro_rules! handlebars_helpers {
    ( $handlebars:expr ) => {
        if cfg!(feature = "helper_inc") {
            handlebars_helper!(inc: |x: i64| x + 1);

            $handlebars.register_helper("inc", Box::new(inc));
        }

        if cfg!(feature = "helper_dec") {
            handlebars_helper!(dec: |x: i64| x - 1);

            $handlebars.register_helper("dec", Box::new(dec));
        }

        if cfg!(feature = "helper_eq_str") {
            handlebars_helper!(eq_str: |x: str, y: str| x == y);

            $handlebars.register_helper("eq_str", Box::new(eq_str));
        }

        if cfg!(feature = "helper_ne_str") {
            handlebars_helper!(ne_str: |x: str, y: str| x != y);

            $handlebars.register_helper("ne_str", Box::new(ne_str));
        }
    };
}

#[cfg(all(not(debug_assertions), not(feature = "helper")))]
macro_rules! handlebars_helpers {
    ($handlebars:expr) => {};
}
