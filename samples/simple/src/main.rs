#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::*;
use tracing::debug;

pub struct App;

impl Runnable for App {
  fn settings() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
      .with_debug_info(DebugInfo::Shown)
      .with_polling(Polling::Poll)
  }

  fn new(_foxy: &Foxy) -> Self {
    Self {}
  }

  fn update(&mut self, _foxy: &Foxy, event: &FoxyEvent) {
    if let FoxyEvent::Input(InputEvent::Keyboard(..)) = event {
      debug!("UPDATE: {:?}", event)
    }
  }
}

fn main() -> FoxyResult<()> {
  start_debug_logging_session!();

  App::run()
}
