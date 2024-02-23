#![deny(unsafe_op_in_unsafe_fn)]

use std::time::Duration;

use quanta::Instant;
use thiserror::Error;
use tracing::*;

use crate::ring_buffer::RingBuffer;

mod ring_buffer;

pub mod game_loop;
pub mod stopwatch;
pub mod timer;

#[derive(Debug, Clone, Copy)]
pub struct Time {
  start_time: Instant,
  delta_time: Duration,
  tick_delta_time: Duration,
  average_delta_time: Duration,
}

#[allow(unused)]
impl Time {
  pub fn since_start(&self) -> Duration {
    Instant::now() - self.start_time
  }

  pub fn delta(&self) -> &Duration {
    &self.delta_time
  }

  pub fn delta_secs(&self) -> f64 {
    self.delta_time.as_secs_f64()
  }

  pub fn delta_tick(&self) -> &Duration {
    &self.tick_delta_time
  }

  pub fn delta_tick_secs(&self) -> f64 {
    self.tick_delta_time.as_secs_f64()
  }

  pub fn average_delta(&self) -> &Duration {
    &self.average_delta_time
  }

  pub fn average_delta_secs(&self) -> f64 {
    self.average_delta_time.as_secs_f64()
  }

  pub fn now(&self) -> Instant {
    Instant::now()
  }
}

#[derive(Debug)]
pub struct TimeSettings {
  pub tick_rate: f64,
  pub bail_threshold: u32,
  pub max_samples: usize,
}

impl Default for TimeSettings {
  fn default() -> Self {
    Self {
      tick_rate: 128.0,
      bail_threshold: 1024,
      max_samples: 128,
    }
  }
}

impl TimeSettings {
  pub fn build(&self) -> EngineTime {
    EngineTime::new(self.tick_rate, self.bail_threshold, self.max_samples)
  }
}

pub struct EngineTime {
  tick_rate: f64,
  tick_time: Duration,
  lag_time: Duration,
  step_count: u32,
  bail_threshold: u32,

  start_time: Instant,

  previous_frame: Instant,
  current_frame: Instant,
  delta_time: Duration,

  tick_previous_frame: Instant,
  tick_current_frame: Instant,
  tick_delta_time: Duration,

  frame_times: RingBuffer<Duration>,
}

impl Default for EngineTime {
  fn default() -> Self {
    const TICK_RATE: f64 = 128.0;
    let tick_time: Duration = Duration::from_secs_f64(1. / TICK_RATE);
    const BAIL_THRESHOLD: u32 = 1024;
    Self {
      tick_rate: TICK_RATE,
      tick_time,
      lag_time: Default::default(),
      step_count: 0,
      bail_threshold: BAIL_THRESHOLD,
      start_time: Instant::now(),
      previous_frame: Instant::now(),
      current_frame: Instant::now(),
      delta_time: Default::default(),
      tick_previous_frame: Instant::now(),
      tick_current_frame: Instant::now(),
      tick_delta_time: Default::default(),
      frame_times: RingBuffer::new(100),
    }
  }
}

impl EngineTime {
  pub fn new(tick_rate: f64, bail_threshold: u32, max_samples: usize) -> Self {
    Self {
      tick_rate,
      bail_threshold,
      frame_times: RingBuffer::new(max_samples),
      ..Default::default()
    }
  }

  pub fn with_tick_rate(mut self, tick_rate: f64) -> Self {
    self.tick_rate = tick_rate;
    self
  }

  pub fn with_bail_threshold(mut self, bail_threshold: u32) -> Self {
    self.bail_threshold = bail_threshold;
    self
  }

  pub fn time(&self) -> Time {
    Time {
      start_time: self.start_time,
      delta_time: self.delta_time,
      tick_delta_time: self.tick_delta_time,
      average_delta_time: self.average_delta(),
    }
  }

  fn average_delta(&self) -> Duration {
    self
      .frame_times
      .iter()
      .sum::<Duration>()
      .checked_div(self.frame_times.len() as u32)
      .unwrap_or_default()
  }

  pub fn next_tick(&mut self) -> Result<bool, (bool, TimeError)> {
    self.update();
    self.should_do_tick().map(|_| {
      self.tick();
      true
    })
  }

  pub fn update(&mut self) {
    self.current_frame = Instant::now();
    self.delta_time = self.current_frame - self.previous_frame;
    self.previous_frame = self.current_frame;
    self.lag_time += self.delta_time;
    self.step_count = 0;

    self.frame_times.push(self.delta_time);
  }

  pub fn tick(&mut self) {
    self.tick_current_frame = Instant::now();
    self.tick_delta_time = self.tick_current_frame - self.tick_previous_frame;
    self.tick_previous_frame = self.tick_current_frame;
    self.lag_time -= self.tick_time;
    self.step_count += 1;
  }

  /// Returns `TimeError::TickOverflow` when struggling to catch up with tick
  /// rate.
  pub fn should_do_tick(&self) -> Result<bool, (bool, TimeError)> {
    let decision = self.should_do_tick_unchecked();
    if self.step_count >= self.bail_threshold {
      Err((decision, TimeError::TickOverflow))
    } else {
      Ok(decision)
    }
  }

  /// Ignores tick rate overflows (without panicking)
  pub fn should_do_tick_unchecked(&self) -> bool {
    self.lag_time >= self.tick_time && self.step_count < self.bail_threshold
  }
}

#[derive(Error, Debug)]
pub enum TimeError {
  #[error("struggling to catch up with tick rate")]
  TickOverflow,
}
