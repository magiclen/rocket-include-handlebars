/// Used in the fairing of `Template` to include Handlebars files into your executable binary file. You need to specify each file's name and its path.
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! handlebars_resources_initialize {
    ( $handlebars:expr, $($name:expr, $path:expr), * $(,)* ) => {
        use std::fs;

        if cfg!(debug_assertions) {
            $handlebars.clear_templates();
        }

        $(
            $handlebars.register_template_file($name, $path).unwrap();
        )*
    };
}

/// Used in the fairing of `Template` to include Handlebars files into your executable binary file. You need to specify each file's name and its path.
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! handlebars_resources_initialize {
    ( $handlebars:expr, $($name:expr, $path:expr), * $(,)* ) => {
        use std::fs;

        if cfg!(debug_assertions) {
            $handlebars.clear_templates();
        }

        $(
            $handlebars.register_template_string($name, include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))).unwrap();
        )*
    };
}

/// Used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
#[macro_export]
macro_rules! handlebars_response {
    ( $name:expr, $data:expr ) => {
        {
            use ::rocket_include_handlebars::{HandlebarsResponse, EtagIfNoneMatch};

            HandlebarsResponse::build_from_template(
                EtagIfNoneMatch {
                    etag: None
                },
                None,
                true,
                $name,
                $data,
            ).unwrap()
        }
    };
    ( $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        {
            use ::rocket_include_handlebars::{HandlebarsResponse, EtagIfNoneMatch};

            HandlebarsResponse::build_from_template(
                $etag_if_none_match,
                None,
                true,
                $name,
                $data,
            ).unwrap()
        }
    };
}

/// This macro can be used to wrap a `HandlebarsResponse` and its constructor, and use a **key** to staticize its HTML and ETag in memory.
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! handlebars_response_static {
    ( $cm:expr, $key:expr, $gen:block ) => {
        {
            drop(&$cm);
            drop(&$key);
            $gen
        }
    };
    ( $etag_if_none_match:expr, $cm:expr, $key:expr, $gen:block ) => {
        {
            drop(&$etag_if_none_match);
            drop(&$cm);
            drop(&$key);
            $gen
        }
    };
}

/// This macro can be used to wrap a `HandlebarsResponse` and its constructor, and use a **key** to staticize its HTML and ETag in memory.
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! handlebars_response_static {
    ( $cm:expr, $key:expr, $gen:block ) => {
        {
            let contains = $cm.cache_table.lock().unwrap().contains_key($key.as_str());

            if contains {
                HandlebarsResponse::build_from_cache(
                    EtagIfNoneMatch {
                        etag: None
                    },
                    $key.as_str()
                )
            } else {
                let res = $gen;

                let cache = res.get_html_and_etag(&$cm).unwrap();

                $cm.cache_table.lock().unwrap().insert($key, cache);

                res
            }
        }
    };
    ( $etag_if_none_match:expr, $cm:expr, $key:expr, $gen:block ) => {
        {
            let contains = $cm.cache_table.lock().unwrap().contains_key($key.as_str());

            if contains {
                HandlebarsResponse::build_from_cache(
                    $etag_if_none_match,
                    $key.as_str()
                )
            } else {
                let res = $gen;

                let cache = res.get_html_and_etag(&$cm).unwrap();

                $cm.cache_table.lock().unwrap().insert($key, cache);

                res
            }
        }
    };
}