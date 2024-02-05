use winit::event::Event;

use super::{builder::FoxyCreateInfo, framework::Framework, state::Foxy, FoxyResult};

#[allow(unused)]
pub trait Runnable<T: 'static + Send + Sync> {
  fn new(foxy: &mut Foxy) -> Self;

  fn start(&mut self, foxy: &mut Foxy) {}

  fn fixed_update(&mut self, foxy: &mut Foxy, event: &Option<Event<T>>) {}

  fn update(&mut self, foxy: &mut Foxy, event: &Option<Event<T>>) {}

  fn late_update(&mut self, foxy: &mut Foxy, event: &Option<Event<T>>) {}

  fn stop(&mut self, foxy: &mut Foxy) -> bool {
    true
  }

  fn delete(mut self)
  where
    Self: Sized,
  {
  }

  fn foxy() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
  }

  /// You don't want to override this method. It's implemented as a simple wrapper around the Framework::run() method.
  fn run() -> FoxyResult<()>
  where
    Self: Sized,
  {
    Framework::with_events::<Self>(Self::foxy())?.run()
  }
}
