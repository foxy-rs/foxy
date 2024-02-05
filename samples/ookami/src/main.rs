#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::{
  prelude::{
    winit::{
      dpi::{PhysicalSize, Size},
      event::{Event, WindowEvent},
    },
    *,
  },
  window::WindowCreateInfo,
};
use tracing::debug;

pub struct App;

impl Runnable<()> for App {
  fn foxy() -> FoxyCreateInfo {
    FoxyCreateInfo {
      window: WindowCreateInfo {
        inner_size: Some(Size::Physical(PhysicalSize {
          width: 800,
          height: 450,
        })),
        ..Default::default()
      },
      ..Default::default()
    }
    .with_debug_info(DebugInfo::Shown)
    .with_polling(Polling::Poll)
  }

  fn new(_foxy: &mut Foxy) -> Self {
    Self {}
  }

  fn update(&mut self, _foxy: &mut Foxy, event: &Option<Event<()>>) {
    if let Some(Event::WindowEvent {
      event: WindowEvent::KeyboardInput { event, .. },
      ..
    }) = event
    {
      debug!("UPDATE: {:?}", event)
    }
  }
}

fn main() -> FoxyResult<()> {
  if let Some(session) = debug_logging_session_ex!(
    ("foxy", Some(LogLevel::Trace)),
    ("foxy_window", Some(LogLevel::Trace)),
    ("foxy_renderer", Some(LogLevel::Trace)),
    ("foxy_vulkan", Some(LogLevel::Trace)),
    ("foxy_utils", Some(LogLevel::Trace)),
    ("ookami", Some(LogLevel::Trace))
  ) {
    session.with_line_numbers(true).with_file_names(true).start();
  }

  App::run()
}
