use std::io::Cursor;

use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
};

use crate::{EntityTag, EtagIfNoneMatch};

#[derive(Debug)]
enum HandlebarsResponseInner {
    Html { content: String, etag: EntityTag<'static> },
    NotModified(Option<String>),
}

#[derive(Debug)]
/// To respond HTML.
pub struct HandlebarsResponse {
    inner: HandlebarsResponseInner,
}

impl HandlebarsResponse {
    #[inline]
    pub(crate) fn build_not_cache<S: Into<String>>(
        content: S,
        etag: EntityTag<'static>,
    ) -> HandlebarsResponse {
        HandlebarsResponse {
            inner: HandlebarsResponseInner::Html {
                content: content.into(),
                etag,
            },
        }
    }

    #[doc(hidden)]
    #[inline]
    pub const fn not_modified() -> HandlebarsResponse {
        HandlebarsResponse {
            inner: HandlebarsResponseInner::NotModified(None)
        }
    }

    #[inline]
    pub(crate) fn not_modified_with_etag(etag: &EntityTag<'_>) -> Self {
        Self {
            inner: HandlebarsResponseInner::NotModified(Some(etag.to_string()))
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn check_etag(self, etag_if_none_match: &EtagIfNoneMatch<'_>) -> Self {
        match &self.inner {
            HandlebarsResponseInner::Html {
                etag, ..
            } if etag_if_none_match.weak_eq(etag) => Self::not_modified_with_etag(etag),
            _ => self,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn weak_eq(&self, etag_if_none_match: &EtagIfNoneMatch<'_>) -> bool {
        matches!(&self.inner, HandlebarsResponseInner::Html { etag, .. } if etag_if_none_match.weak_eq(etag))
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for HandlebarsResponse {
    #[inline]
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        let mut response = Response::build();

        match self.inner {
            HandlebarsResponseInner::Html {
                content,
                etag,
            } => {
                response.raw_header("Content-Type", "text/html; charset=utf-8");
                response.raw_header(EntityTag::HEADER_NAME, etag.to_string());
                response.sized_body(content.len(), Cursor::new(content));
            },
            HandlebarsResponseInner::NotModified(etag) => {
                response.status(Status::NotModified);

                if let Some(etag) = etag {
                    response.raw_header(EntityTag::HEADER_NAME, etag);
                }
            },
        }

        response.ok()
    }
}
