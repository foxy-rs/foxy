use winit::event::Event;

use super::state::Foxy;

// pub type WindowEvent<T: 'static + Send + Sync> = Event<T>;

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
}
