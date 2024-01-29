#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::time::Duration;

use foxy::prelude::*;
use tracing::*;

fn main() {
  if let Some(session) = debug_logging_session_ex!(
    ("foxy", Some(LogLevel::Trace)),
    ("foxy_window", Some(LogLevel::Trace)),
    ("foxy_renderer", Some(LogLevel::Trace)),
    ("foxy_vulkan", Some(LogLevel::Trace)),
    ("foxy_types", Some(LogLevel::Trace)),
    ("foxy_utils", Some(LogLevel::Trace)),
    ("ookami", Some(LogLevel::Trace))
  ) {
    session.with_line_numbers(true).with_file_names(true).start();
  }

  let framework = Framework::builder()
    .with_title("Foxy Renderer")
    .with_size(800, 450)
    .build_unwrap();

  for stage in framework {
    match stage {
      Stage::Initialize => {
        debug!("oh, hi");
      }
      Stage::FixedUpdate { foxy } => {
        foxy.append_fps_every(Duration::from_millis(300));
      }
      Stage::Update { message, .. } => match message {
        WindowMessage::None | WindowMessage::Other { .. } | WindowMessage::Mouse(MouseMessage::Cursor) => {}
        _ => debug!("UPDATE: {:?}", message),
      },
      _ => {}
    }
  }
}
