use std::time::Duration;

use foxy_util::time::timer::Timer;
use foxy_window::window::Window;
use tracing::info;

use super::time::Time;

pub struct FoxyFramework {
  pub time: Time,
  pub window: Window,
  fps_timer: Timer,
}

impl FoxyFramework {
  pub fn new(time: Time, window: Window) -> Self {
    Self {
      time,
      window,
      fps_timer: Timer::new(),
    }
  }

  pub fn append_fps_every(&mut self, duration: Duration) {
    if self.fps_timer.has_elapsed(duration) {
      let fps = 1.0 / self.time.average_delta_secs();
      self.window.set_title(&format!("{}: {:.2}", self.window.title(), fps));
    }
  }

  pub fn append_ft_every(&mut self, duration: Duration) {
    if self.fps_timer.has_elapsed(duration) {
      self.window.set_title(&format!("{}: {:.6}", self.window.title(), self.time.average_delta_secs()));
    }
  }

  pub fn print_fps(&self) {
    let fps = 1.0 / self.time.average_delta_secs();
    info!("{}", format!("{}: {:.2}", self.window.title(), fps));
  }
}
