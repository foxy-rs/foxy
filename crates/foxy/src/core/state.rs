use std::{sync::Arc, time::Duration};

use egui::{epaint::Shadow, RawInput, Rounding, Visuals};
use ezwin::{
  prelude::{ButtonState, Key, KeyState, Mouse},
  window::Window,
};
use foxy_time::Time;

pub struct Foxy {
  pub(crate) time: Time,
  pub(crate) window: Arc<Window>,
  pub(crate) egui_context: egui::Context,
}

impl Foxy {
  pub fn new(time: Time, window: Arc<Window>) -> Self {
    let egui_context = egui::Context::default();

    const BORDER_RADIUS: f32 = 6.0;

    let visuals = Visuals {
      window_rounding: Rounding::same(BORDER_RADIUS),
      menu_rounding: Rounding::same(BORDER_RADIUS),
      window_shadow: Shadow::NONE,
      ..Default::default()
    };

    egui_context.set_visuals(visuals);

    Self {
      time,
      window,
      egui_context,
    }
  }

  pub fn delta_time(&self) -> Duration {
    self.time.delta().clone()
  }

  pub fn average_delta_time(&self) -> Duration {
    self.time.average_delta().clone()
  }

  pub fn window(&self) -> &Arc<Window> {
    &self.window
  }

  pub fn key(&self, key: Key) -> KeyState {
    self.window.key(key)
  }

  pub fn mouse(&self, mouse: Mouse) -> ButtonState {
    self.window.mouse(mouse)
  }

  pub fn shift(&self) -> ButtonState {
    self.window.shift()
  }

  pub fn ctrl(&self) -> ButtonState {
    self.window.ctrl()
  }

  pub fn alt(&self) -> ButtonState {
    self.window.alt()
  }

  pub fn win(&self) -> ButtonState {
    self.window.win()
  }

  pub fn take_egui_raw_input(&self) -> RawInput {
    RawInput::default()
  }
}
