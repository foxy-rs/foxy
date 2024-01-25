#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::*;
use tracing::*;

fn main() {
  if cfg!(debug_assertions) {
    logging_session!().start();
  }

  let mut app = Foxy::builder().with_title("Simple").with_size(800, 450).build_unwrap();

  while let Some(stage) = app.poll() {
    match stage {
      Lifecycle::FixedUpdate { .. } => debug!("FixedUpdate"),
      Lifecycle::Update { .. } => debug!("Update"),
      _ => {}
    }
  }
}
