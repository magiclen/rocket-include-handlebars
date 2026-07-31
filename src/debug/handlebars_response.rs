use std::io::Cursor;

use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
};

use crate::{EntityTag, EtagIfNoneMatch};

#[derive(Debug)]
struct HandlebarsResponseInner {
    content: String,
    etag:    EntityTag<'static>,
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
        etag: EntityTag<'static>,
    ) -> HandlebarsResponse {
        HandlebarsResponse {
            inner: Some(HandlebarsResponseInner {
                content: content.into(),
                etag,
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
    pub fn weak_eq(&self, etag_if_none_match: &EtagIfNoneMatch<'_>) -> bool {
        self.inner.as_ref().is_some_and(|inner| etag_if_none_match.weak_eq(&inner.etag))
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for HandlebarsResponse {
    #[inline]
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        let mut response = Response::build();

        if let Some(inner) = self.inner {
            response.raw_header("Content-Type", "text/html; charset=utf-8");
            response.raw_header("Etag", inner.etag.to_string());

            response.sized_body(inner.content.len(), Cursor::new(inner.content));
        } else {
            response.status(Status::NotModified);
        }

        response.ok()
    }
}
