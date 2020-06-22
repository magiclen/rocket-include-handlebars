#[cfg(feature = "helper")]
macro_rules! handlebars_helpers {
    ( $handlebars:expr ) => {
        #[cfg(feature = "helper_inc")]
        {
            handlebars_helper!(inc: |x: i64| x + 1);

            $handlebars.register_helper("inc", Box::new(inc));
        }

        #[cfg(feature = "helper_dec")]
        {
            handlebars_helper!(dec: |x: i64| x - 1);

            $handlebars.register_helper("dec", Box::new(dec));
        }

        #[cfg(feature = "helper_eq_str")]
        {
            handlebars_helper!(eq_str: |x: str, y: str| x == y);

            $handlebars.register_helper("eq_str", Box::new(eq_str));
        }

        #[cfg(feature = "helper_ne_str")]
        {
            handlebars_helper!(ne_str: |x: str, y: str| x != y);

            $handlebars.register_helper("ne_str", Box::new(ne_str));
        }

        #[cfg(feature = "helper_lookup_map")]
        {
            handlebars_helper!(lookup_map: |map: object, key: str| map.get(key).cloned().unwrap_or($crate::handlebars::JsonValue::Null));

            $handlebars.register_helper("lookup_map", Box::new(lookup_map));
        }

        #[cfg(feature = "helper_lookup_array")]
        {
            handlebars_helper!(lookup_array: |array: array, index: u64| array.get(index as usize).cloned().unwrap_or($crate::handlebars::JsonValue::Null));

            $handlebars.register_helper("lookup_array", Box::new(lookup_array));
        }
    };
}

#[cfg(all(not(debug_assertions), not(feature = "helper")))]
macro_rules! handlebars_helpers {
    ($handlebars:expr) => {};
}
