#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::{egui::{self, Align2}, prelude::*};
use tracing::{debug, warn};

pub struct App {
  x: u32,
}

impl Runnable for App {
  fn settings() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
      .with_size(800, 600)
      .with_debug_info(DebugInfo::Shown)
      .with_polling(Polling::Poll)
  }

  fn new(_foxy: &Foxy) -> Self {
    Self { x: 0 }
  }

  fn input(&mut self, foxy: &Foxy, event: &InputEvent) {
    if let InputEvent::Mouse(button, state) = event {
      debug!(
        "UPDATE | {:?}: {:?} + {:?}",
        button,
        state,
        foxy.read().input().shift().is_pressed()
      )
    }
  }

  fn gui(&mut self, foxy: &Foxy, egui: &foxy::egui::Context) {
    egui::Window::new("Settings")
      .default_open(false)
      .default_size((50.0, 50.0))
      .resizable(false)
      .anchor(Align2::LEFT_BOTTOM, (5.0, -5.0))
      .show(egui, |ui| {
        if ui.button("Test").clicked() {
          debug!("PRESSED");
        }

        let slider = ui.add(egui::Slider::new(&mut self.x, 1..=10)).on_hover_text("Slider");
        if slider.changed() {
          debug!("x: {}", self.x);
        }
      });
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
