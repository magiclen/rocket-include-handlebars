use std::sync::{Arc, Mutex};

use crate::EntityTag;

#[cfg(debug_assertions)]
use crate::ReloadableHandlebars;

#[cfg(not(debug_assertions))]
use crate::Handlebars;

use crate::lru_time_cache::LruCache;

/// To monitor the state of Handlebars.
#[cfg(debug_assertions)]
#[derive(Educe)]
#[educe(Debug)]
#[allow(clippy::type_complexity)]
pub struct HandlebarsContextManager {
    pub handlebars: Mutex<ReloadableHandlebars>,
    #[educe(Debug(ignore))]
    cache_table: Mutex<LruCache<String, (Arc<str>, Arc<EntityTag>)>>,
}

/// To monitor the state of Handlebars.
#[cfg(not(debug_assertions))]
#[derive(Educe)]
#[educe(Debug)]
#[allow(clippy::type_complexity)]
pub struct HandlebarsContextManager {
    pub handlebars: Handlebars<'static>,
    #[educe(Debug(ignore))]
    cache_table: Mutex<LruCache<String, (Arc<str>, Arc<EntityTag>)>>,
}

impl HandlebarsContextManager {
    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) fn new(
        handlebars: Mutex<ReloadableHandlebars>,
        cache_capacity: usize,
    ) -> HandlebarsContextManager {
        HandlebarsContextManager {
            handlebars,
            cache_table: Mutex::new(LruCache::with_capacity(cache_capacity)),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) fn new(
        handlebars: Handlebars<'static>,
        cache_capacity: usize,
    ) -> HandlebarsContextManager {
        HandlebarsContextManager {
            handlebars,
            cache_table: Mutex::new(LruCache::with_capacity(cache_capacity)),
        }
    }

    #[inline]
    /// Clear cache.
    pub fn clear_cache(&self) {
        self.cache_table.lock().unwrap().clear();
    }

    #[inline]
    /// Check if a cache key exists.
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {
        self.cache_table.lock().unwrap().get(key.as_ref()).is_some()
    }

    #[inline]
    /// Get the cache by a specific key.
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<(Arc<str>, Arc<EntityTag>)> {
        self.cache_table
            .lock()
            .unwrap()
            .get(key.as_ref())
            .map(|(html, etag)| (html.clone(), etag.clone()))
    }

    #[inline]
    /// Insert a cache.
    pub fn insert<S: Into<String>>(
        &self,
        key: S,
        cache: (Arc<str>, Arc<EntityTag>),
    ) -> Option<(Arc<str>, Arc<EntityTag>)> {
        self.cache_table.lock().unwrap().insert(key.into(), cache)
    }
}
