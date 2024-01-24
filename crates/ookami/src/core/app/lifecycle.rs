use foxy_window::prelude::*;

use super::time::Time;

pub trait Lifecycle {
  fn new() -> Option<Self>
  where
    Self: Sized;

  fn start(&mut self, _: &Time, _: &mut Window) {}

  fn early_update(&mut self, _: &Time, _: &mut Window, _: &WindowMessage) {}

  fn fixed_update(&mut self, _: &Time, _: &mut Window) {}

  fn update(&mut self, _: &Time, _: &mut Window, _: &WindowMessage) {}

  fn stop(&mut self, _: &Time, _: &mut Window) {}
}