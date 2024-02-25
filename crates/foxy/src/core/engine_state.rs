use std::sync::{Arc, RwLockReadGuard};

use ezwin::window::{input::Input, Window};
use foxy_time::{EngineTime, Time};

pub struct Foxy {
  pub(crate) time: EngineTime,
  pub(crate) window: Arc<Window>,
}

impl Foxy {
  pub fn new(time: EngineTime, window: Arc<Window>) -> Self {
    Self { time, window }
  }

  pub fn time(&self) -> Time {
    self.time.time()
  }

  pub fn window(&self) -> Arc<Window> {
    self.window.clone()
  }

  pub fn input(&self) -> RwLockReadGuard<Input> {
    self.window.input()
  }
}
