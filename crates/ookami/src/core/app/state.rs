use ezwin::prelude::*;
use tracing::*;

use crate::core::time::Time;

#[derive(Debug)]
pub struct AppState {
    pub time: Time,
}

// TODO: Move AppWindow reference into Messages
impl AppState {
    pub fn new() -> Self {
        Self {
            time: Time::new(128.0, 1024),
        }
    }

    pub fn start(&mut self, _window: &mut Window) {
        trace!("START");
    }

    pub fn early_update(&mut self, _window: &mut Window, _msg: &WindowMessage) {
        // trace!("EARLY_UPDATE");
    }

    pub fn fixed_update(&mut self, window: &mut Window) {
        let fps = 1.0 / self.time.average_delta_secs();
        window.set_title(&format!("{}: {:.2}", window.title(), fps));
        // trace!("FIXED_UPDATE: [{}]", fps)
        // trace!("FIXED_UPDATE");
    }

    pub fn update(&mut self, _window: &mut Window, msg: &WindowMessage) {
        // let fps = 1.0 / self.time.average_delta_secs();
        // trace!("UPDATE: {:?}", _msg)
        match msg {
            WindowMessage::Empty | WindowMessage::Other { .. } | WindowMessage::Mouse(MouseMessage::Cursor) => {}
            _ => debug!("UPDATE: {:?}", msg),
        }
        // trace!("UPDATE");
    }

    pub fn stop(&mut self, _window: &mut Window) {
        trace!("STOP");
    }
}
