use std::time::Duration;

use quanta::Instant;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Timer {
  start_of_lap: Instant,
}

#[allow(dead_code)]
impl Timer {
  pub fn new() -> Self {
    Self {
      start_of_lap: Instant::now(),
    }
  }

  pub fn has_elapsed(&mut self, duration: Duration) -> bool {
    let now = Instant::now();
    let is_elapsed = now - self.start_of_lap >= duration;
    if is_elapsed {
      self.start_of_lap = now;
    }
    is_elapsed
  }
}

impl Default for Timer {
  fn default() -> Self {
    Self::new()
  }
}
