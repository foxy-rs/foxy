use foxy_utils::time::{EngineTime, Time};
use foxy_window::window::Window;

pub struct Foxy {
  pub(crate) time: EngineTime,
  window: Window,
}

impl Foxy {
  pub fn new(time: EngineTime, window: Window) -> Self {
    Self { time, window }
  }

  pub fn time(&self) -> Time {
    self.time.time()
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn window_mut(&mut self) -> &mut Window {
    &mut self.window
  }
}
