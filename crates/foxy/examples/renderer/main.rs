#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use foxy::prelude::*;
use std::time::Duration;
use tracing::*;

fn main() {
  if cfg!(debug_assertions) {
    logging_session_ex!(
      ("foxy_window", Some(LogLevel::Trace)),
      ("foxy_vulkan", Some(LogLevel::Trace)),
      ("foxy_renderer", Some(LogLevel::Trace)),
      ("renderer", Some(LogLevel::Trace))
    )
    .start();
    log_lib_info!();
  }

  let mut foxy = Foxy::builder()
    .with_title("Foxy Renderer")
    .with_size(800, 450)
    .build()
    .unwrap_or_else(|e| panic!("{e}"));

  let mut fps_timer = Timer::new(Duration::from_secs_f64(0.33));

  while let Some(message) = foxy.poll() {
    match message {
      Lifecycle::EarlyUpdate { .. } => {}
      Lifecycle::FixedUpdate { .. } => {
        if fps_timer.is_elapsed() {
          let fps = 1.0 / foxy.time().average_delta_secs();
          foxy
            .window()
            .set_title(&format!("{}: {:.2}", foxy.window().title(), fps));
        }
      }
      Lifecycle::Update { message } => match message {
        WindowMessage::None | WindowMessage::Other { .. } | WindowMessage::Mouse(MouseMessage::Cursor) => {}
        _ => debug!("UPDATE: {:?}", message),
      },
      _ => {}
    }
  }
}
