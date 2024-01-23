use ookami::prelude::*;
use std::time::Duration;
use tracing::*;

pub struct State {
  fps_timer: Timer,
  message_timer: Timer,
}

impl Lifecycle for State {
  fn new() -> Option<Self> {
    Some(Self {
      fps_timer: Timer::new(Duration::from_secs_f64(0.33)),
      message_timer: Timer::new(Duration::from_secs_f64(0.5)),
    })
  }

  fn early_update(&mut self, time: &Time, _window: &mut Window, _msg: &WindowMessage) {
    if self.message_timer.is_elapsed() {
      trace!("MESSAGE");
    }
    // trace!("EARLY_UPDATE");
  }

  fn fixed_update(&mut self, time: &Time, window: &mut Window) {
    if self.fps_timer.is_elapsed() {
      let fps = 1.0 / time.average_delta_secs();
      window.set_title(&format!("{}: {:.2}", window.title(), fps));
    }
  }

  fn update(&mut self, time: &Time, _window: &mut Window, msg: &WindowMessage) {
    // let fps = 1.0 / self.time.average_delta_secs();
    // trace!("UPDATE: {:?}", _msg)
    match msg {
      WindowMessage::Empty | WindowMessage::Other { .. } | WindowMessage::Mouse(MouseMessage::Cursor) => {}
      _ => debug!("UPDATE: {:?}", msg),
    }
    // trace!("UPDATE");
  }
}
