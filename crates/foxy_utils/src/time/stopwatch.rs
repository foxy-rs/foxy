use std::time::Duration;

use quanta::Instant;

#[allow(dead_code)]
pub struct Stopwatch {
  start_time: Instant,
}

impl Default for Stopwatch {
  fn default() -> Self {
    Self::new()
  }
}

#[allow(dead_code)]
impl Stopwatch {
  pub fn new() -> Self {
    Self {
      start_time: Instant::now(),
    }
  }

  pub fn elapsed(&self) -> Duration {
    Instant::now() - self.start_time
  }
}
