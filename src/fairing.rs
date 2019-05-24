#[cfg(debug_assertions)]
use std::sync::{Mutex, MutexGuard};

use crate::rocket::Rocket;
#[cfg(debug_assertions)]
use crate::rocket::State;
#[cfg(debug_assertions)]
use crate::rocket::request::Request;
use crate::rocket::fairing::{Fairing, Info, Kind};
#[cfg(debug_assertions)]
use crate::rocket::data::Data;

#[cfg(debug_assertions)]
use crate::ReloadableHandlebars;

#[cfg(not(debug_assertions))]
use crate::Handlebars;

use crate::HandlebarsContextManager;

const FAIRING_NAME: &'static str = "Handlebars";

/// The fairing of `HandlebarsResponse`.
#[cfg(debug_assertions)]
pub struct HandlebarsResponseFairing {
    pub(crate) custom_callback: Box<Fn(&mut MutexGuard<ReloadableHandlebars>) + Send + Sync + 'static>
}

/// The fairing of `HandlebarsResponse`.
#[cfg(not(debug_assertions))]
pub struct HandlebarsResponseFairing {
    pub(crate) custom_callback: Box<Fn(&mut Handlebars) + Send + Sync + 'static>
}

impl Fairing for HandlebarsResponseFairing {
    #[cfg(debug_assertions)]
    fn info(&self) -> Info {
        Info {
            name: FAIRING_NAME,
            kind: Kind::Attach | Kind::Request,
        }
    }

    #[cfg(not(debug_assertions))]
    fn info(&self) -> Info {
        Info {
            name: FAIRING_NAME,
            kind: Kind::Attach,
        }
    }

    #[cfg(debug_assertions)]
    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let handlebars = Mutex::new(ReloadableHandlebars::new());

        (self.custom_callback)(&mut handlebars.lock().unwrap());

        let state = HandlebarsContextManager::new(handlebars);

        Ok(rocket.manage(state))
    }

    #[cfg(not(debug_assertions))]
    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let mut handlebars = Handlebars::new();

        (self.custom_callback)(&mut handlebars);

        let state = HandlebarsContextManager::new(handlebars);

        Ok(rocket.manage(state))
    }

    #[cfg(debug_assertions)]
    fn on_request(&self, req: &mut Request, _data: &Data) {
        let cm = req.guard::<State<HandlebarsContextManager>>().expect("HandlebarsContextManager registered in on_attach");

        cm.handlebars.lock().unwrap().reload_if_needed().unwrap();
    }
}