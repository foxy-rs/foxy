use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use foxy_utils::{time::{EngineTime, Time}, types::handle::Handle};
use foxy_window::window::Window;

pub struct Foxy {
  pub(crate) time: EngineTime,
  window: Handle<Window>,
}

impl Foxy {
  pub fn new(time: EngineTime, window: Handle<Window>) -> Self {
    Self { time, window }
  }

  pub fn time(&self) -> Time {
    self.time.time()
  }

  pub fn window(&self) -> RwLockReadGuard<Window> {
    self.window.get()
  }

  pub fn window_mut(&mut self) -> RwLockWriteGuard<Window> {
    self.window.get_mut()
  }
}
