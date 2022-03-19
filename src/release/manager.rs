use std::sync::Arc;
use std::sync::Mutex;

use handlebars::Handlebars;
use lru_time_cache::LruCache;
use serde::Serialize;

use crate::functions::compute_data_etag;
use crate::{EntityTag, EtagIfNoneMatch};

use super::HandlebarsResponse;

#[allow(clippy::type_complexity)]
/// To monitor the state of Handlebars.
#[derive(Educe)]
#[educe(Debug)]
pub struct HandlebarsContextManager {
    pub handlebars: Handlebars<'static>,
    #[educe(Debug(ignore))]
    cache_table: Mutex<LruCache<String, (Arc<str>, Arc<EntityTag<'static>>)>>,
}

impl HandlebarsContextManager {
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

    /// Build a `HandlebarsResponse`.
    #[inline]
    pub fn build<S: AsRef<str>, V: Serialize>(
        &self,
        etag_if_none_match: &EtagIfNoneMatch<'_>,
        minify: bool,
        name: S,
        context: V,
    ) -> HandlebarsResponse {
        self.handlebars
            .render(name.as_ref(), &context)
            .map(|html| {
                let etag = compute_data_etag(html.as_bytes());

                if etag_if_none_match.weak_eq(&etag) {
                    HandlebarsResponse::not_modified()
                } else {
                    let html = if minify {
                        html_minifier::minify(html).unwrap()
                    } else {
                        html
                    };

                    HandlebarsResponse::build_not_cache(html, &etag)
                }
            })
            .unwrap()
    }

    /// Build a `HandlebarsResponse`.
    #[inline]
    pub fn build_from_cache<K: AsRef<str>>(
        &self,
        etag_if_none_match: &EtagIfNoneMatch<'_>,
        key: K,
    ) -> Option<HandlebarsResponse> {
        self.cache_table.lock().unwrap().get(key.as_ref()).map(|(html, etag)| {
            if etag_if_none_match.weak_eq(etag) {
                HandlebarsResponse::not_modified()
            } else {
                HandlebarsResponse::build_cache(html.clone(), etag)
            }
        })
    }

    /// Render a template.
    #[inline]
    pub fn render<S: AsRef<str>, V: Serialize>(&self, name: S, context: V) -> String {
        self.handlebars.render(name.as_ref(), &context).unwrap()
    }

    /// Clear cache.
    #[inline]
    pub fn clear_cache(&self) {
        self.cache_table.lock().unwrap().clear();
    }

    /// Check if a cache key exists.
    #[inline]
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {
        self.cache_table.lock().unwrap().get(key.as_ref()).is_some()
    }

    /// Get the cache by a specific key.
    #[inline]
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<(Arc<str>, Arc<EntityTag<'static>>)> {
        self.cache_table
            .lock()
            .unwrap()
            .get(key.as_ref())
            .map(|(html, etag)| (html.clone(), etag.clone()))
    }

    /// Insert a cache.
    #[inline]
    pub fn insert<S: Into<String>>(
        &self,
        key: S,
        cache: (Arc<str>, Arc<EntityTag<'static>>),
    ) -> Option<(Arc<str>, Arc<EntityTag<'static>>)> {
        self.cache_table.lock().unwrap().insert(key.into(), cache)
    }
}
