use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    pub cache_table: Mutex<HashMap<Arc<str>, (Arc<str>, Arc<EntityTag>)>>,
}

/// To monitor the state of Handlebars.
#[cfg(not(debug_assertions))]
#[derive(Debug)]
pub struct HandlebarsContextManager {
    pub handlebars: Handlebars,
    pub cache_table: Mutex<HashMap<Arc<str>, (Arc<str>, Arc<EntityTag>)>>,
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

    #[inline]
    /// Check if a cache key exists.
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {
        self.cache_table.lock().unwrap().contains_key(key.as_ref())
    }

    #[inline]
    /// Insert a cache.
    pub fn insert<S: Into<Arc<str>>>(&self, key: S, cache: (Arc<str>, Arc<EntityTag>)) -> Option<(Arc<str>, Arc<EntityTag>)> {
        self.cache_table.lock().unwrap().insert(key.into(), cache)
    }
}