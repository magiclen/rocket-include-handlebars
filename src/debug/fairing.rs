use std::sync::{Mutex, MutexGuard, PoisonError};

use rocket::data::Data;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::request::Request;
use rocket::{Build, Rocket};

use super::{HandlebarsContextManager, HandlebarsResponse, ReloadableHandlebars};

const FAIRING_NAME: &str = "Handlebars (Debug)";

/// The fairing of `HandlebarsResponse`.
pub struct HandlebarsResponseFairing {
    pub(crate) custom_callback:
        Box<dyn Fn(&mut MutexGuard<ReloadableHandlebars>) -> usize + Send + Sync + 'static>,
}

#[rocket::async_trait]
impl Fairing for HandlebarsResponseFairing {
    #[inline]
    fn info(&self) -> Info {
        Info {
            name: FAIRING_NAME,
            kind: Kind::Ignite | Kind::Request,
        }
    }

    #[inline]
    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
        let handlebars = Mutex::new(ReloadableHandlebars::new());

        let cache_capacity =
            (self.custom_callback)(&mut handlebars.lock().unwrap_or_else(PoisonError::into_inner));

        let state = HandlebarsContextManager::new(handlebars, cache_capacity);

        Ok(rocket.manage(state))
    }

    #[inline]
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let cm = req
            .rocket()
            .state::<HandlebarsContextManager>()
            .expect("HandlebarsContextManager registered in on_attach");

        cm.handlebars.lock().unwrap_or_else(PoisonError::into_inner).reload_if_needed().unwrap();
    }
}

impl HandlebarsResponse {
    /// Create the fairing of `HandlebarsResponse`.
    #[inline]
    pub fn fairing<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut MutexGuard<ReloadableHandlebars>) + Send + Sync + 'static, {
        let f = Box::new(f);

        HandlebarsResponseFairing {
            custom_callback: Box::new(move |handlebars| {
                f(handlebars);

                crate::DEFAULT_CACHE_CAPACITY
            }),
        }
    }

    /// Create the fairing of `HandlebarsResponse` and set the cache capacity.
    #[inline]
    pub fn fairing_cache<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut MutexGuard<ReloadableHandlebars>) -> usize + Send + Sync + 'static, {
        HandlebarsResponseFairing {
            custom_callback: Box::new(f),
        }
    }
}
