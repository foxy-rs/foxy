#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::*;
use tracing::debug;

pub struct App;

impl Runnable for App {
  fn settings() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
      .with_size(800, 450)
      .with_debug_info(DebugInfo::Shown)
      .with_polling(Polling::Poll)
  }

  fn new(_foxy: &mut Foxy) -> Self {
    Self {}
  }

  fn input(&mut self, foxy: &mut Foxy, event: &InputEvent) {
    if let InputEvent::Mouse(button, state) = event {
      debug!("UPDATE | {:?}: {:?} + {:?}", button, state, foxy.input().shift().is_pressed())
    }
  }
}

fn main() -> FoxyResult<()> {
  start_logging();
  App::run()
}

fn start_logging() {
  if let Some(session) = debug_logging_session_ex!(
    ("foxy", Some(LogLevel::Trace)),
    ("foxy_renderer", Some(LogLevel::Trace)),
    ("foxy_utils", Some(LogLevel::Trace)),
    ("ookami", Some(LogLevel::Trace))
  ) {
    session.with_line_numbers(true).with_file_names(true).start();
  }
}
