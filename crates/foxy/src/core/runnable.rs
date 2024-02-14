use egui::Context;

use super::{
  builder::FoxyCreateInfo,
  engine_state::Foxy,
  event::{FoxyEvent, InputEvent, WindowEvent},
  framework::Framework,
  FoxyResult,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Flow {
  Exit,
  Continue,
}

#[allow(unused)]
pub trait Runnable {
  fn new(foxy: &mut Foxy) -> Self;

  fn start(&mut self, foxy: &mut Foxy) {}

  fn fixed_update(&mut self, foxy: &mut Foxy, event: &FoxyEvent) {}

  fn input(&mut self, foxy: &mut Foxy, event: &InputEvent) {}

  fn update(&mut self, foxy: &mut Foxy, event: &FoxyEvent) {}

  fn late_update(&mut self, foxy: &mut Foxy, event: &FoxyEvent) {}

  fn window(&mut self, foxy: &mut Foxy, event: &WindowEvent) {}

  fn gui(&mut self, foxy: &mut Foxy, egui: Context) {}

  fn stop(&mut self, foxy: &mut Foxy) -> Flow {
    Flow::Exit
  }

  fn delete(mut self)
  where
    Self: Sized,
  {
  }

  fn settings() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
  }

  /// ## You don't want to override this method. It's implemented as a simple wrapper around the Framework::run() method.
  fn run() -> FoxyResult<()>
  where
    Self: Sized,
  {
    Framework::new::<Self>(Self::settings())?.run()
  }
}
