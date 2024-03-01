use std::sync::{Arc, RwLock, RwLockWriteGuard};

use egui::{epaint::Shadow, RawInput, Rounding, Visuals};
use foxy_time::{EngineTime, Time};
use winit::{event::MouseButton, keyboard::KeyCode, window::Window};

use super::input::{
  state::{KeyState, MouseState},
  Input,
};

pub struct Foxy {
  pub(crate) time: EngineTime,
  pub(crate) window: Arc<Window>,
  pub(crate) input: Arc<RwLock<Input>>,
  pub(crate) egui_context: egui::Context,
}

impl Foxy {
  pub fn new(time: EngineTime, window: Arc<Window>) -> Self {
    let input = Arc::new(RwLock::new(Input::new()));

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
      input,
      egui_context,
    }
  }

  pub fn time(&self) -> Time {
    self.time.time()
  }

  pub fn window(&self) -> &Arc<Window> {
    &self.window
  }

  pub fn key(&self, key: KeyCode) -> KeyState {
    self.input.read().unwrap().key(key)
  }

  pub fn mouse(&self, mouse: MouseButton) -> MouseState {
    self.input.read().unwrap().mouse(mouse)
  }

  pub fn shift(&self) -> MouseState {
    self.input.read().unwrap().shift()
  }

  pub fn ctrl(&self) -> MouseState {
    self.input.read().unwrap().ctrl()
  }

  pub fn alt(&self) -> MouseState {
    self.input.read().unwrap().alt()
  }

  pub fn win(&self) -> MouseState {
    self.input.read().unwrap().win()
  }

  pub(crate) fn input(&self) -> RwLockWriteGuard<Input> {
    self.input.write().unwrap()
  }

  pub fn take_egui_raw_input(&self) -> RawInput {
    RawInput::default()
  }
}
