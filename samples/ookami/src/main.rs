#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::{
  winit::{
    dpi::{LogicalSize, Size},
    event::{KeyEvent, WindowEvent},
  },
  *,
};
use tracing::debug;

pub struct App;

impl Runnable for App {
  fn foxy() -> FoxyCreateInfo {
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

  fn update(&mut self, _foxy: &mut Foxy, event: &Option<WindowEvent>) {
    if let Some(
      WindowEvent::KeyboardInput {
        event: KeyEvent {
          physical_key, state, ..
        },
        ..
      },
    ) = event
    {
      debug!("UPDATE | {:?}: {:?}", physical_key, state)
    } else if let Some(
      WindowEvent::MouseInput {
        button,
        state,
        ..
      },
    ) = event
    {
      debug!("UPDATE | {:?}: {:?}", button, state)
    }
  }
}

fn main() -> FoxyResult<()> {
  if let Some(session) = debug_logging_session_ex!(
    ("foxy", Some(LogLevel::Trace)),
    ("foxy_renderer", Some(LogLevel::Trace)),
    ("foxy_utils", Some(LogLevel::Trace)),
    ("ookami", Some(LogLevel::Trace))
  ) {
    session.with_line_numbers(true).with_file_names(true).start();
  }

  App::run()
}