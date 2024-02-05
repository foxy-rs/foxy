use std::sync::Arc;

use foxy_utils::time::{EngineTime, Time};
use winit::window::Window;

pub struct Foxy {
  pub(crate) time: EngineTime,
  pub window: Arc<Window>,
}

impl Foxy {
  pub fn new(time: EngineTime, window: Arc<Window>) -> Self {
    Self { time, window }
  }

  pub fn time(&self) -> Time {
    self.time.time()
  }
}
