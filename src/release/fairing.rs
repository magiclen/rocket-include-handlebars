use handlebars::Handlebars;
use rocket::{
    fairing::{Fairing, Info, Kind},
    Build, Rocket,
};

use super::{HandlebarsContextManager, HandlebarsResponse};
use crate::functions::add_helpers;

const FAIRING_NAME: &str = "Handlebars";

/// The fairing of `HandlebarsResponse`.
pub struct HandlebarsResponseFairing {
    pub(crate) custom_callback: Box<dyn Fn(&mut Handlebars) -> usize + Send + Sync + 'static>,
}

#[rocket::async_trait]
impl Fairing for HandlebarsResponseFairing {
    #[inline]
    fn info(&self) -> Info {
        Info {
            name: FAIRING_NAME, kind: Kind::Ignite
        }
    }

    #[inline]
    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
        let mut handlebars = Handlebars::new();

        add_helpers(&mut handlebars);

        let cache_capacity = (self.custom_callback)(&mut handlebars);

        let state = HandlebarsContextManager::new(handlebars, cache_capacity);

        Ok(rocket.manage(state))
    }
}

impl HandlebarsResponse {
    /// Create the fairing of `HandlebarsResponse`.
    #[inline]
    pub fn fairing<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut Handlebars) + Send + Sync + 'static, {
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
        F: Fn(&mut Handlebars) -> usize + Send + Sync + 'static, {
        HandlebarsResponseFairing {
            custom_callback: Box::new(f)
        }
    }
}
