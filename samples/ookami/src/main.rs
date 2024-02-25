#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::prelude::*;
use tracing::debug;

pub struct App;

impl Runnable for App {
  fn settings() -> FoxySettings {
    FoxySettings::default()
      .with_window(WindowSettings::default().with_size((800, 450)).with_flow(Flow::Poll))
      .with_debug_info(DebugInfo::Shown)
  }

  fn new(_foxy: &mut Foxy) -> Self {
    Self {}
  }

  fn update(&mut self, foxy: &mut Foxy, message: &Message) {
    if let Message::Keyboard { key, state, .. } = message {
      debug!("UPDATE | {:?}: {:?} + {:?}", key, state, foxy.input().shift().is_pressed())
    }

    if let Message::Mouse(MouseMessage::Button { button, state, .. }) = message {
      debug!("UPDATE | {:?}: {:?} + {:?}", button, state, foxy.input().shift().is_pressed())
    }
  }

  fn delete(self)
  where
    Self: Sized,
  {
    debug!("Delete!")
  }
}

fn main() -> FoxyResult<()> {
  start_logging();
  App::run()
}

fn start_logging() {
  if let Some(session) = debug_logging_session_ex!(
    ("foxy", Some(LogLevel::Trace)),
    ("foxy_renderer", Some(LogLevel::Trace)),
    ("foxy_utils", Some(LogLevel::Trace)),
    ("ookami", Some(LogLevel::Trace))
  ) {
    session.with_line_numbers(true).with_file_names(true).start();
  }
}
