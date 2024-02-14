use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use egui::{epaint::Shadow, Context, Visuals};
use foxy_utils::time::{EngineTime, Time};
use winit::window::Window;

use super::input::Input;

#[derive(Clone)]
pub struct Foxy(Arc<RwLock<State>>);

impl Foxy {
  pub fn new(state: State) -> Self {
    Self(Arc::new(RwLock::new(state)))
  }

  pub fn read(&self) -> RwLockReadGuard<State> {
    self.0.read().expect("reader panicked")
  }

  pub fn write(&self) -> RwLockWriteGuard<State> {
    self.0.write().expect("reader panicked")
  }
}

pub struct State {
  pub(crate) engine_time: EngineTime,
  pub(crate) window: Arc<Window>,
  pub(crate) egui_state: egui_winit::State,
  pub(crate) egui_context: Context,
  pub(crate) input: Input,
}

impl State {
  pub fn new(engine_time: EngineTime, window: Arc<Window>) -> Self {
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
      engine_time,
      window,
      egui_state,
      egui_context,
      input: Input::new(),
    }
  }

  pub fn time(&self) -> Time {
    self.engine_time.time()
  }

  pub fn window(&self) -> &Arc<Window> {
    &self.window
  }

  pub fn input(&self) -> &Input {
    &self.input
  }
}
