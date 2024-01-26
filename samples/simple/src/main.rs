#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::*;
use tracing::*;

fn main() {
  start_debug_logging_session!();

  let foxy = Foxy::builder().with_title("Simple").with_size(800, 450).build_unwrap();

  for stage in foxy {
    match stage {
      Stage::FixedUpdate { .. } => debug!("FixedUpdate"),
      Stage::Update { .. } => debug!("Update"),
      _ => {}
    }
  }
}
