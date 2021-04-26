extern crate rc_u8_reader;

use std::io::Cursor;
use std::sync::Arc;

use rc_u8_reader::ArcU8Reader;

use crate::rocket::http::Status;
use crate::rocket::request::Request;
use crate::rocket::response::{self, Responder, Response};
use crate::EntityTag;

#[derive(Debug)]
enum HandlebarsResponseInner {
    NotCache {
        content: String,
        etag: String,
    },
    Cache {
        content: Arc<str>,
        etag: String,
    },
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
                etag: etag.to_string(),
            }),
        }
    }

    #[inline]
    pub(crate) fn build_cache(content: Arc<str>, etag: &EntityTag<'static>) -> HandlebarsResponse {
        HandlebarsResponse {
            inner: Some(HandlebarsResponseInner::Cache {
                content,
                etag: etag.to_string(),
            }),
        }
    }

    #[inline]
    pub(crate) const fn not_modified() -> HandlebarsResponse {
        HandlebarsResponse {
            inner: None,
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
                }
                HandlebarsResponseInner::Cache {
                    content,
                    etag,
                } => {
                    response.raw_header("Etag", etag);
                    response.sized_body(content.len(), ArcU8Reader::new(content));
                }
            }
        } else {
            response.status(Status::NotModified);
        }

        response.ok()
    }
}
