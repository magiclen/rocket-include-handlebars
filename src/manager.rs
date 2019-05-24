use std::collections::HashMap;
use std::sync::Mutex;

use crate::EntityTag;

#[cfg(debug_assertions)]
use crate::ReloadableHandlebars;

#[cfg(not(debug_assertions))]
use crate::Handlebars;

/// To monitor the state of Handlebars.
#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct HandlebarsContextManager {
    pub handlebars: Mutex<ReloadableHandlebars>,
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
    pub(crate) fn new(handlebars: Mutex<ReloadableHandlebars>) -> HandlebarsContextManager {
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