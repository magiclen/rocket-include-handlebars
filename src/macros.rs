/// Used for including HBS files into your executable binary file. You need to specify each file's ID and its path.
#[macro_export]
macro_rules! handlebars_resources_initialize {
    ( $($id:expr, $path:expr), * $(,)* ) => {
        lazy_static_include_str_vec!(HANDLEBARS_REG_DATA $(, $path)* );

        lazy_static! {
            static ref HANDLEBARS_REG: ::rocket_include_handlebars::handlebars::Handlebars = {
                {
                    use ::rocket_include_handlebars::handlebars::Handlebars;

                    let mut reg = Handlebars::new();

                    let mut p = 0usize;

                    $(
                        {
                            let template = HANDLEBARS_REG_DATA[p];

                            p += 1;

                            reg.register_template_string($id, template).unwrap();
                        }
                    )*

                    handlebars_helper!(inc: |x: i64| x + 1);

                    handlebars_helper!(dec: |x: i64| x - 1);

                    handlebars_helper!(eq_str: |x: str, y: str| x == y);

                    handlebars_helper!(ne_str: |x: str, y: str| x != y);

                    reg.register_helper("inc", Box::new(inc));
                    reg.register_helper("dec", Box::new(dec));
                    reg.register_helper("eq_str", Box::new(eq_str));
                    reg.register_helper("ne_str", Box::new(ne_str));

                    reg
                }
            };

            static ref HANDLEBARS_STATIC: std::sync::Mutex<std::collections::HashMap<String, (String, ::rocket_include_handlebars::EntityTag)>> = {
                std::sync::Mutex::new(std::collections::HashMap::new())
            };
        }
    };
}


/// Used for retrieving and rendering the file you input through the macro `handlebars_resources_initialize!` as a `HandlebarsResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
#[macro_export]
macro_rules! handlebars_response {
    ( $id:expr, $data:expr ) => {
        {
            use ::rocket_include_handlebars::HandlebarsResponse;

            let html = HANDLEBARS_REG.render($id, $data).unwrap();

            HandlebarsResponse{
                html,
                client_etag: ::rocket_include_handlebars::EtagIfNoneMatch {etag: None},
                etag: None,
                minify: true,
            }
        }
    };
    ( $etag_if_none_match:expr, $id:expr, $data:expr ) => {
        {
            use ::rocket_include_handlebars::HandlebarsResponse;

            let html = HANDLEBARS_REG.render($id, $data).unwrap();

            HandlebarsResponse{
                html,
                client_etag: $etag_if_none_match,
                etag: None,
                minify: true,
            }
        }
    };
}

/// This macro can be used to wrap a `HandlebarsResponse` and its constructor, and use a **key** to staticize its HTML and ETag in memory.
#[macro_export]
macro_rules! handlebars_response_static {
    ( $key:expr, $gen:block ) => {
        {
            if HANDLEBARS_STATIC.lock().unwrap().contains_key($key.as_str()) {
                let map = HANDLEBARS_STATIC.lock().unwrap();
                let (html, etag) = map.get($key.as_str()).unwrap();
                HandlebarsResponse{
                    html: html.clone(),
                    client_etag: ::rocket_include_handlebars::EtagIfNoneMatch {etag: None},
                    etag: Some(etag.clone()),
                    minify: false,
                }
            } else{
                let mut res = $gen;

                if res.minify {
                    res.html = ::rocket_include_handlebars::html_minifier::minify(&res.html).unwrap();
                    res.minify = false;
                }

                let etag = match res.etag {
                    Some(etag) => etag,
                    None => {
                        let mut crc64ecma = ::rocket_include_handlebars::crc_any::CRC::crc64ecma();
                        crc64ecma.digest(res.html.as_bytes());
                        let crc64 = crc64ecma.get_crc();
                        ::rocket_include_handlebars::EntityTag::new(true, format!("{:X}", crc64))
                    }
                };

                HANDLEBARS_STATIC.lock().unwrap().insert($key, (res.html.clone(), etag.clone()));

                res.etag = Some(etag);

                res
            }
        }
    };
    ( $etag_if_none_match:expr, $key:expr, $gen:block ) => {
        {
            if HANDLEBARS_STATIC.lock().unwrap().contains_key($key.as_str()) {
                let map = HANDLEBARS_STATIC.lock().unwrap();
                let (html, etag) = map.get($key.as_str()).unwrap();
                HandlebarsResponse{
                    html: html.clone(),
                    client_etag: $etag_if_none_match,
                    etag: Some(etag.clone()),
                    minify: false,
                }
            } else{
                let mut res = $gen;

                if res.minify {
                    res.html = ::rocket_include_handlebars::html_minifier::minify(&res.html).unwrap();
                }

                let etag = match res.etag {
                    Some(etag) => etag,
                    None => {
                        let mut crc64ecma = ::rocket_include_handlebars::crc_any::CRC::crc64ecma();
                        crc64ecma.digest(res.html.as_bytes());
                        let crc64 = crc64ecma.get_crc();
                        ::rocket_include_handlebars::EntityTag::new(true, format!("{:X}", crc64))
                    }
                };

                HANDLEBARS_STATIC.lock().unwrap().insert($key, (res.html.clone(), etag.clone()));

                HandlebarsResponse{
                    html: res.html,
                    client_etag: $etag_if_none_match,
                    etag: Some(etag),
                    minify: false,
                }
            }
        }
    };
}