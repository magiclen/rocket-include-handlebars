use std::sync::{Mutex, PoisonError};

use serde::Serialize;

use super::{HandlebarsResponse, ReloadableHandlebars};
use crate::{functions::compute_data_etag, EtagIfNoneMatch};

/// To monitor the state of Handlebars.
#[derive(Educe)]
#[educe(Debug)]
pub struct HandlebarsContextManager {
    pub handlebars: Mutex<ReloadableHandlebars>,
}

impl HandlebarsContextManager {
    #[inline]
    pub(crate) fn new(
        handlebars: Mutex<ReloadableHandlebars>,
        _cache_capacity: usize,
    ) -> HandlebarsContextManager {
        HandlebarsContextManager {
            handlebars,
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
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .render(name.as_ref(), &context)
            .map(|html| {
                let etag = compute_data_etag(html.as_bytes());

                if etag_if_none_match.weak_eq(&etag) {
                    HandlebarsResponse::not_modified()
                } else {
                    let html = if minify { html_minifier::minify(html).unwrap() } else { html };

                    HandlebarsResponse::build_not_cache(html, &etag)
                }
            })
            .unwrap()
    }

    /// Render a template.
    #[inline]
    pub fn render<S: AsRef<str>, V: Serialize>(&self, name: S, context: V) -> String {
        self.handlebars
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .render(name.as_ref(), &context)
            .unwrap()
    }
}
