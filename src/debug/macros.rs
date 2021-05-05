/// Used in the fairing of `HandlebarsResponse` to include Handlebars files into your executable binary file. You need to specify each file's name and its path relative to the directory containing the manifest of your package. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
#[macro_export]
macro_rules! handlebars_resources_initialize {
    ( $handlebars:expr, $($name:expr => $path:expr), * $(,)* ) => {
        {
            use ::std::fs;
            use ::std::collections::HashSet;

            let mut set: HashSet<&'static str> = HashSet::new();

            $(
                if set.contains($name) {
                    panic!("The name `{}` is duplicated.", $name);
                } else {
                    $handlebars.register_template_file($name, $crate::manifest_dir_macros::not_directory_path!($path)).unwrap();

                    set.insert($name);
                }
            )*
        }
    };
}

/// Used for wrapping a `HandlebarsResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
#[macro_export]
macro_rules! handlebars_response_cache {
    ($cm:expr, $etag_if_none_match:expr, $key:expr, $gen:block) => {{
        #[allow(unused_variables)]
        let __a = &$cm;
        #[allow(unused_variables)]
        let __a = &$key;

        let res = $gen;

        if res.weak_eq(&$etag_if_none_match) {
            $crate::HandlebarsResponse::not_modified()
        } else {
            res
        }
    }};
}
