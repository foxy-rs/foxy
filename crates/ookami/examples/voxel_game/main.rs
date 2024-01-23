#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use eztracing::prelude::*;
use ookami::prelude::*;

use self::state::State;

mod state;

fn main() {
  if cfg!(debug_assertions) {
    logging_session_ex!(("foxy_window", Some(LogLevel::Trace)), ("foxy_vulkan", Some(LogLevel::Trace)))
      // .with_file_names(true)
      // .with_line_numbers(true)
      .start();
    log_lib_info!();
  }

  AppBuilder::new()
    .with_title("Ookami Renderer")
    .with_size(800, 450)
    .run::<State>();
}
