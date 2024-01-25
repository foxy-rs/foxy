#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use std::time::Duration;

use eztracing::prelude::*;
use ookami::prelude::*;
use tracing::*;

mod state;

fn main() {
  if cfg!(debug_assertions) {
    logging_session_ex!(
      ("foxy_window", Some(LogLevel::Trace)),
      ("foxy_vulkan", Some(LogLevel::Trace)),
      ("foxy_renderer", Some(LogLevel::Trace)),
      ("voxel_game", Some(LogLevel::Trace))
    )
    .start();
    log_lib_info!();
  }

  let mut app = App::builder()
    .with_title("Ookami Renderer")
    .with_size(800, 450)
    .build()
    .unwrap_or_else(|e| panic!("{e}"));

  let mut fps_timer = Timer::new(Duration::from_secs_f64(0.33));
  let mut message_timer = Timer::new(Duration::from_secs_f64(0.5));

  while let Some(message) = app.poll() {
    match message {
      Lifecycle::EarlyUpdate { .. } => {
        if message_timer.is_elapsed() {
          trace!("MESSAGE");
        }
      }
      Lifecycle::FixedUpdate { .. } => {
        if fps_timer.is_elapsed() {
          let fps = 1.0 / app.time().average_delta_secs();
          app.window().set_title(&format!("{}: {:.2}", app.window().title(), fps));
        }
      }
      Lifecycle::Update { message } => {
        match message {
          Some(WindowMessage::Empty | WindowMessage::Other { .. } | WindowMessage::Mouse(MouseMessage::Cursor)) | None => {}
          Some(_) => debug!("UPDATE: {:?}", message),
        }
      }
      _ => {}
    }
  }
}
