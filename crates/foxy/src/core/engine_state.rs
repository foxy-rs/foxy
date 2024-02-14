use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use egui::{epaint::Shadow, style::HandleShape, Context, Rounding, Visuals};
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
  pub(crate) egui_context: Context,
  pub(crate) input: Input,
}

impl State {
  pub fn new(engine_time: EngineTime, window: Arc<Window>) -> Self {
    let egui_context = Context::default();

    Self {
      engine_time,
      window,
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
