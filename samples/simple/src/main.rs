#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::*;
use tracing::*;

fn main() {
  start_debug_logging_session!();

  let mut x: u32 = 0;

  let foxy = Framework::builder()
    .with_title("Simple Foxy App")
    .with_size(800, 450)
    .build_unwrap();

  for stage in foxy {
    match stage {
      Stage::FixedUpdate { .. } => {
        x = x.wrapping_add(1);
        debug!("FixedUpdate")
      }
      Stage::Update { .. } => {
        debug!("Update {x}")
      }
      _ => {}
    }
  }
}
