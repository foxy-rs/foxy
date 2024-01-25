use foxy::prelude::*;
use tracing::debug;

fn main() {
  if cfg!(debug_assertions) {
    logging_session_ex!(("simple", Some(LogLevel::Trace))).start();
  }

  let mut app = Foxy::builder()
    .with_title("Simple")
    .with_size(800, 450)
    .build()
    .unwrap_or_else(|e| panic!("{e}"));

  while let Some(message) = app.wait() {
    match message {
      Lifecycle::Entering => debug!("Entering"),
      Lifecycle::Update { .. } => debug!("Update"),
      Lifecycle::Exiting => debug!("Exiting"),
      _ => {}
    }
  }
}
