use std::sync::Arc;

use foxy_utils::time::{EngineTime, Time};
use winit::window::Window;

use super::input::Input;

pub struct Foxy {
  pub(crate) time: EngineTime,
  pub(crate) window: Arc<Window>,
  pub(crate) input: Input,
}

impl Foxy {
  pub fn new(time: EngineTime, window: Arc<Window>) -> Self {
    Self {
      time,
      window,
      input: Input::new(),
    }
  }

  pub fn time(&self) -> Time {
    self.time.time()
  }

  pub fn window(&self) -> Arc<Window> {
    self.window.clone()
  }

  pub fn input(&self) -> &Input {
    &self.input
  }
}
