#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::*;
use std::time::Duration;
use tracing::*;

fn main() {
  start_debug_logging_session_ex!(
    ("foxy", Some(LogLevel::Trace)),
    ("foxy_window", Some(LogLevel::Trace)),
    ("foxy_renderer", Some(LogLevel::Trace)),
    ("foxy_vulkan", Some(LogLevel::Trace)),
    ("foxy_types", Some(LogLevel::Trace)),
    ("foxy_utils", Some(LogLevel::Trace)),
    ("ookami", Some(LogLevel::Trace))
  );

  let lifecycle = Lifecycle::builder()
    .with_title("Foxy Renderer")
    .with_size(800, 450)
    .build_unwrap();

  for stage in lifecycle {
    match stage {
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
