use std::time::Duration;

use quanta::Instant;

#[allow(dead_code)]
pub struct Timer {
  start_of_lap: Instant,
  duration: Duration,
}

#[allow(dead_code)]
impl Timer {
  pub fn new(duration: Duration) -> Self {
    Self {
      start_of_lap: Instant::now(),
      duration,
    }
  }

  pub fn is_elapsed(&mut self) -> bool {
    let now = Instant::now();
    let is_elapsed = now - self.start_of_lap >= self.duration;
    if is_elapsed {
      self.start_of_lap = now;
    }
    is_elapsed
  }
}
