use tracing::trace;

use crate::core::time::Time;

use super::window::message::Message;

pub struct AppState {
    pub time: Time,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            time: Time::new(128.0, 1024),
        }
    }

    pub fn start(&mut self) {
        trace!("START");
    }

    pub fn early_update(&mut self, _event: Option<&Message>) {
        // trace!("EARLY_UPDATE");
    }

    pub fn fixed_update(&mut self, _event: Option<&Message>) {
        // trace!("FIXED_UPDATE");
    }

    pub fn update(&mut self, _event: Option<&Message>) {
        // trace!("UPDATE");
    }

    pub fn stop(&mut self) {
        trace!("STOP");
    }
}
