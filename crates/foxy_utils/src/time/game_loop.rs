use super::{EngineTime, Time};

pub struct GameLoop {
  pub time: EngineTime,
  pub start: Box<dyn FnMut(Time)>,
  pub early_update: Box<dyn FnMut(Time)>,
  pub fixed_update: Box<dyn FnMut(Time)>,
  pub update: Box<dyn FnMut(Time)>,
  pub stop: Box<dyn FnMut(Time)>,
}

impl Default for GameLoop {
  fn default() -> Self {
    Self {
      time: EngineTime::default(),
      start: Box::new(|_| {}),
      early_update: Box::new(|_| {}),
      fixed_update: Box::new(|_| {}),
      update: Box::new(|_| {}),
      stop: Box::new(|_| {}),
    }
  }
}

#[allow(unused)]
impl GameLoop {
  pub fn run(mut self, should_continue: impl Fn() -> bool) {
    (self.start)(self.time.time());
    while (should_continue)() {
      self.time.update();
      (self.early_update)(self.time.time());
      while self.time.should_do_tick_unchecked() {
        self.time.tick();
        (self.fixed_update)(self.time.time());
      }
      (self.update)(self.time.time());
    }
    (self.stop)(self.time.time());
  }
}
