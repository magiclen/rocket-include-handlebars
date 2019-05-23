use std::collections::HashMap;
use std::sync::Mutex;

use crate::{EntityTag, Handlebars};

/// To monitor the state of Handlebars.
#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct HandlebarsContextManager {
    pub handlebars: Mutex<Handlebars>,
    pub cache_table: Mutex<HashMap<String, (String, EntityTag)>>,
}

/// To monitor the state of Handlebars.
#[cfg(not(debug_assertions))]
#[derive(Debug)]
pub struct HandlebarsContextManager {
    pub handlebars: Handlebars,
    pub cache_table: Mutex<HashMap<String, (String, EntityTag)>>,
}

impl HandlebarsContextManager {
    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) fn new(handlebars: Mutex<Handlebars>) -> HandlebarsContextManager {
        HandlebarsContextManager {
            handlebars,
            cache_table: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) fn new(handlebars: Handlebars) -> HandlebarsContextManager {
        HandlebarsContextManager {
            handlebars,
            cache_table: Mutex::new(HashMap::new()),
        }
    }
}