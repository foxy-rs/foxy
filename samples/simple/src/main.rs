#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::{prelude::*, winit::event::WindowEvent};
use tracing::debug;

pub struct App;

impl Runnable for App {
  fn settings() -> FoxySettings {
    FoxySettings::default()
      .with_window(WindowSettings::default().with_should_wait(false))
      .with_debug_info(DebugInfo::Shown)
  }

  fn new(_foxy: &mut Foxy) -> Self {
    Self {}
  }

  fn update(&mut self, _foxy: &mut Foxy, event: Option<&WindowEvent>) {
    if let Some(WindowEvent::KeyboardInput { .. }) = event {
      debug!("UPDATE: {:?}", event)
    }
  }
}

fn main() -> FoxyResult<()> {
  start_debug_logging_session!();

  App::run()
}
