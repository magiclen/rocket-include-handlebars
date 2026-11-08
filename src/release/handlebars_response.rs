use std::{io::Cursor, sync::Arc};

use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
};

use crate::EntityTag;

#[derive(Debug)]
enum HandlebarsResponseInner {
    NotCache { content: String, etag: EntityTag<'static> },
    Cache { content: Arc<str>, etag: Arc<EntityTag<'static>> },
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
            inner: HandlebarsResponseInner::NotCache {
                content: content.into(),
                etag,
            },
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn build_cache(content: Arc<str>, etag: Arc<EntityTag<'static>>) -> HandlebarsResponse {
        HandlebarsResponse {
            inner: HandlebarsResponseInner::Cache {
                content,
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

    #[doc(hidden)]
    #[inline]
    pub fn not_modified_with_etag(etag: &EntityTag<'_>) -> Self {
        Self {
            inner: HandlebarsResponseInner::NotModified(Some(etag.to_string()))
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn into_html_and_etag(self) -> Option<(Arc<str>, Arc<EntityTag<'static>>)> {
        self.into_cache_parts().ok()
    }

    #[doc(hidden)]
    #[inline]
    pub fn into_cache_parts(self) -> Result<(Arc<str>, Arc<EntityTag<'static>>), Self> {
        match self.inner {
            HandlebarsResponseInner::NotCache {
                content,
                etag,
            } => Ok((Arc::from(content), Arc::new(etag))),
            HandlebarsResponseInner::Cache {
                content,
                etag,
            } => Ok((content, etag)),
            HandlebarsResponseInner::NotModified(_) => Err(self),
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for HandlebarsResponse {
    #[inline]
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        // `Arc<str>` does not implement `AsRef<[u8]>`, so wrap it to let `Cursor` read the cached HTML without copying it.
        #[derive(Debug)]
        struct SharedHtml(Arc<str>);

        impl AsRef<[u8]> for SharedHtml {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                self.0.as_bytes()
            }
        }

        let mut response = Response::build();

        match self.inner {
            HandlebarsResponseInner::NotCache {
                content,
                etag,
            } => {
                response.raw_header("Content-Type", "text/html; charset=utf-8");
                response.raw_header(EntityTag::HEADER_NAME, etag.to_string());
                response.sized_body(content.len(), Cursor::new(content));
            },
            HandlebarsResponseInner::Cache {
                content,
                etag,
            } => {
                response.raw_header("Content-Type", "text/html; charset=utf-8");
                response.raw_header(EntityTag::HEADER_NAME, etag.to_string());
                response.sized_body(content.len(), Cursor::new(SharedHtml(content)));
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
