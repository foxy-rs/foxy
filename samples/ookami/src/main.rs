#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::{winit::event::WindowEvent, *};
use tracing::debug;

// fn main() {
//   if let Some(session) = debug_logging_session_ex!(
//     ("foxy", Some(LogLevel::Trace)),
//     ("foxy_window", Some(LogLevel::Trace)),
//     ("foxy_renderer", Some(LogLevel::Trace)),
//     ("foxy_vulkan", Some(LogLevel::Trace)),
//     ("foxy_utils", Some(LogLevel::Trace)),
//     ("ookami", Some(LogLevel::Trace))
//   ) {
//     session.with_line_numbers(true).with_file_names(true).start();
//   }

//   let framework = Framework::builder()
//     .with_title("Foxy Renderer")
//     .with_size(800, 450)
//     .with_debug_info(DebugInfo::Shown)
//     .build_unwrap();

//   for stage in framework {
//     if let Stage::Update { message, .. } = stage {
//       if let WindowMessage::Keyboard(..) = message {
//         debug!("UPDATE: {:?}", message)
//       }
//     }
//   }
// }

pub struct App;

impl Runnable<()> for App {
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

  Framework::new::<App>(FoxyCreateInfo::default())?.run()
}
