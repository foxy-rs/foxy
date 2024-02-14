#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::{
  egui::{self, Align2},
  prelude::*,
};
use tracing::{debug, warn};

pub struct App;

impl Runnable for App {
  fn settings() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
      .with_size(800, 450)
      .with_debug_info(DebugInfo::Shown)
      .with_polling(Polling::Poll)
  }

  fn new(_foxy: &mut Foxy) -> Self {
    Self {}
  }

  fn input(&mut self, foxy: &mut Foxy, event: &InputEvent) {
    if let InputEvent::Mouse(button, state) = event {
      debug!("UPDATE | {:?}: {:?} + {:?}", button, state, foxy.input().shift().is_pressed())
    }
  }

  fn gui(&mut self, foxy: &mut Foxy, egui: &foxy::egui::Context) {
    egui::Window::new("Streamline CFD")
        // .vscroll(true)
        .default_open(true)
        .max_width(1000.0)
        .max_height(800.0)
        .default_width(800.0)
        .resizable(true)
        .movable(true)
        .show(egui, |ui| {
            if ui.add(egui::Button::new("Click me")).clicked() {
                warn!("PRESSED")
            }

            ui.label("Slider");
            // ui.add(egui::Slider::new(_, 0..=120).text("age"));
            ui.end_row();

            // proto_scene.egui(ui);
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
