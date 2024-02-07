#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::{
  core::event::{FoxyEvent, InputEvent},
  prelude::{
    winit::dpi::{LogicalSize, Size},
    *,
  },
};
use tracing::debug;

pub struct App;

impl Runnable for App {
  fn settings() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
      .with_size(Size::Logical(LogicalSize {
        width: 800.0,
        height: 450.0,
      }))
      .with_debug_info(DebugInfo::Shown)
      .with_polling(Polling::Poll)
  }

  fn new(_foxy: &mut Foxy) -> Self {
    Self {}
  }

  fn update(&mut self, foxy: &mut Foxy, event: &FoxyEvent) {
    if let FoxyEvent::Input(InputEvent::Mouse(button, state)) = event {
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
