use egui::Context;

use super::{
  builder::FoxyCreateInfo,
  event::{FoxyEvent, InputEvent, WindowEvent},
  foxy_loop::Framework,
  foxy_state::Foxy,
  FoxyResult,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Flow {
  Exit,
  Continue,
}

#[allow(unused)]
pub trait Runnable {
  fn new(foxy: &Foxy) -> Self;

  fn start(&mut self, foxy: &Foxy) {}

  fn fixed_update(&mut self, foxy: &Foxy, event: &FoxyEvent) {}

  fn input(&mut self, foxy: &Foxy, event: &InputEvent) {}

  fn update(&mut self, foxy: &Foxy, event: &FoxyEvent) {}

  fn late_update(&mut self, foxy: &Foxy, event: &FoxyEvent) {}

  fn window(&mut self, foxy: &Foxy, event: &WindowEvent) {}

  fn gui(&mut self, foxy: &Foxy, egui: &Context) {}

  fn stop(&mut self, foxy: &Foxy) -> Flow {
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
