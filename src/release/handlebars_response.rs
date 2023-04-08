use std::{io::Cursor, sync::Arc};

use rc_u8_reader::ArcU8Reader;
use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
};

use crate::EntityTag;

#[derive(Debug)]
enum HandlebarsResponseInner {
    NotCache { content: String, etag: String },
    Cache { content: Arc<str>, etag: String },
}

#[derive(Debug)]
/// To respond HTML.
pub struct HandlebarsResponse {
    inner: Option<HandlebarsResponseInner>,
}

impl HandlebarsResponse {
    #[inline]
    pub(crate) fn build_not_cache<S: Into<String>>(
        content: S,
        etag: &EntityTag<'static>,
    ) -> HandlebarsResponse {
        HandlebarsResponse {
            inner: Some(HandlebarsResponseInner::NotCache {
                content: content.into(),
                etag:    etag.to_string(),
            }),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn build_cache(content: Arc<str>, etag: &EntityTag<'static>) -> HandlebarsResponse {
        HandlebarsResponse {
            inner: Some(HandlebarsResponseInner::Cache {
                content,
                etag: etag.to_string(),
            }),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub const fn not_modified() -> HandlebarsResponse {
        HandlebarsResponse {
            inner: None
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn into_html_and_etag(self) -> Option<(Arc<str>, EntityTag<'static>)> {
        match self.inner {
            Some(HandlebarsResponseInner::NotCache {
                content,
                mut etag,
            }) => {
                etag.remove(etag.len() - 1);
                etag.remove(0);

                let etag = unsafe { EntityTag::with_string_unchecked(false, etag) };

                Some((Arc::from(content), etag))
            },
            Some(HandlebarsResponseInner::Cache {
                content,
                mut etag,
            }) => {
                etag.remove(etag.len() - 1);
                etag.remove(0);

                let etag = unsafe { EntityTag::with_string_unchecked(false, etag) };

                Some((content, etag))
            },
            None => None,
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for HandlebarsResponse {
    #[inline]
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        let mut response = Response::build();

        if let Some(inner) = self.inner {
            response.raw_header("Content-Type", "text/html; charset=utf-8");

            match inner {
                HandlebarsResponseInner::NotCache {
                    content,
                    etag,
                } => {
                    response.raw_header("Etag", etag);
                    response.sized_body(content.len(), Cursor::new(content));
                },
                HandlebarsResponseInner::Cache {
                    content,
                    etag,
                } => {
                    response.raw_header("Etag", etag);
                    response.sized_body(content.len(), ArcU8Reader::new(content));
                },
            }
        } else {
            response.status(Status::NotModified);
        }

        response.ok()
    }
}
