/// Render a registered template as a `HandlebarsResponse`.
/// A `200 OK` response includes `Content-Type`, `Content-Length`, and `ETag`.
/// A `304 Not Modified` response includes the `ETag` and has no body.
/// You can turn off HTML minification.
#[macro_export]
macro_rules! handlebars_response {
    ( $cm:expr, $etag_if_none_match:expr, $name:expr ) => {
        {
            use ::std::collections::HashMap;

            let map: HashMap<u8, u8> = HashMap::new();

            $crate::handlebars_response!($cm, $etag_if_none_match, $name, map)
        }
    };
    ( $cm:expr, $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        $crate::handlebars_response!(enable_minify $cm, $etag_if_none_match, $name, $data)
    };
    ( enable_minify $cm:expr, $etag_if_none_match:expr, $name:expr ) => {
        {
            use ::std::collections::HashMap;

            let map: HashMap<u8, u8> = HashMap::new();

            $crate::handlebars_response!(enable_minify $cm, $etag_if_none_match, $name, map)
        }
    };
    ( enable_minify $cm:expr, $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        $cm.build(
            &$etag_if_none_match,
            true,
            $name,
            &$data,
        )
    };
    ( disable_minify $cm:expr, $etag_if_none_match:expr, $name:expr ) => {
        {
            use ::std::collections::HashMap;

            let map: HashMap<u8, u8> = HashMap::new();

            $crate::handlebars_response!(disable_minify $cm, $etag_if_none_match, $name, map)
        }
    };
    ( disable_minify $cm:expr, $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        $cm.build(
            &$etag_if_none_match,
            false,
            $name,
            &$data,
        )
    };
    ( auto_minify $cm:expr, $etag_if_none_match:expr, $name:expr ) => {
        {
            use ::std::collections::HashMap;

            let map: HashMap<u8, u8> = HashMap::new();

            $crate::handlebars_response!(auto_minify $cm, $etag_if_none_match, $name, map)
        }
    };
    ( auto_minify $cm:expr, $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        $cm.build(&$etag_if_none_match, !cfg!(debug_assertions), $name, &$data)
    };
}

/// Used for generating a fairing for handlebars resources.
#[macro_export]
macro_rules! handlebars_resources_initializer {
    ( $($name:expr => $path:expr), * $(,)* ) => {
        {
            $crate::HandlebarsResponse::fairing(|_handlebars| {
                $crate::handlebars_resources_initialize!(
                    _handlebars,
                    $($name => $path),*
                );
            })
        }
    };
    ( $capacity:expr; $($name:expr => $path:expr), * $(,)*  ) => {
        {
            $crate::HandlebarsResponse::fairing_cache(|_handlebars| {
                $crate::handlebars_resources_initialize!(
                    _handlebars,
                    $($name => $path),*
                );

                $capacity
            })
        }
    };
}
