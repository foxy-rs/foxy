use std::sync::Arc;

use egui::{epaint::Shadow, Context, Visuals};
use foxy_utils::time::{EngineTime, Time};
use winit::window::Window;

use super::input::Input;

pub struct Foxy {
  pub(crate) time: EngineTime,
  pub(crate) window: Arc<Window>,
  pub(crate) egui_state: egui_winit::State,
  pub(crate) egui_context: Context,
  pub(crate) input: Input,
}

impl Foxy {
  pub fn new(time: EngineTime, window: Arc<Window>) -> Self {
    let egui_context = Context::default();

    let id = egui_context.viewport_id();

    const BORDER_RADIUS: f32 = 2.0;

    let visuals = Visuals {
      window_rounding: egui::Rounding::same(BORDER_RADIUS),
      window_shadow: Shadow::NONE,
      // menu_rounding: todo!(),
      ..Default::default()
    };

    egui_context.set_visuals(visuals);

    let egui_state = egui_winit::State::new(egui_context.clone(), id, &window, None, None);

    Self {
      time,
      window,
      egui_state,
      egui_context,
      input: Input::new(),
    }
  }

  pub fn time(&self) -> Time {
    self.time.time()
  }

  pub fn window(&self) -> Arc<Window> {
    self.window.clone()
  }

  pub fn input(&self) -> &Input {
    &self.input
  }
}
