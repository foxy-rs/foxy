use foxy::prelude::*;
use tracing::debug;

fn main() {
  if cfg!(debug_assertions) {
    logging_session!().start();
  }

  let mut app = Foxy::builder()
    .with_title("Simple")
    .with_size(800, 450)
    .build_or_panic();

  while let Some(message) = app.wait() {
    match message {
      Lifecycle::Start => debug!("Start"),
      // Lifecycle::Update { .. } => debug!("Update"),
      Lifecycle::Exiting => debug!("Exiting"),
      _ => {}
    }
  }
}