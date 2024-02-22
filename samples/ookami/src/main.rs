#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use foxy::{
  egui::{self, Align2},
  prelude::*,
  StandardMaterial,
  StaticMesh,
  Vertex,
};
use tracing::*;

pub struct App {
  mesh: StaticMesh,
  x: f32,
}

impl Runnable for App {
  fn new(foxy: &Foxy) -> Self {
    let x = 0.5;
    let mesh = StaticMesh::new(
      &[
        Vertex {
          position: [-x, -x, 0.0],
          color: [1.0, 0.0, 0.0, 1.0],
          uv: [0., 1.],
        },
        Vertex {
          position: [x, -x, 0.0],
          color: [1.0, 1.0, 0.0, 1.0],
          uv: [1., 1.],
        },
        Vertex {
          position: [x, x, 0.0],
          color: [0.0, 1.0, 1.0, 1.0],
          uv: [1., 0.],
        },
        Vertex {
          position: [-x, x, 0.0],
          color: [0.0, 0.0, 1.0, 1.0],
          uv: [0., 0.],
        },
      ],
      Some(&[0, 1, 2, 0, 2, 3]),
      StandardMaterial::new(None),
    );
    Self { x, mesh }
  }

  fn input(&mut self, foxy: &Foxy, event: &InputEvent) {
    if let InputEvent::Mouse(button, state) = event {
      debug!(
        "UPDATE | {:?}: {:?} + {:?}",
        button,
        state,
        foxy.as_ref().input().shift().is_pressed()
      )
    }
  }

  fn update(&mut self, foxy: &Foxy, event: &FoxyEvent) {
    foxy.as_mut().submit_mesh(StaticMesh::new(
      &[
        Vertex {
          position: [-self.x, -self.x, 0.0],
          color: [1.0, 0.0, 0.0, 1.0],
          uv: [0., 1.],
        },
        Vertex {
          position: [self.x, -self.x, 0.0],
          color: [1.0, 1.0, 0.0, 1.0],
          uv: [1., 1.],
        },
        Vertex {
          position: [self.x, self.x, 0.0],
          color: [0.0, 1.0, 1.0, 1.0],
          uv: [1., 0.],
        },
        Vertex {
          position: [-self.x, self.x, 0.0],
          color: [0.0, 0.0, 1.0, 1.0],
          uv: [0., 0.],
        },
      ],
      Some(&[0, 1, 2, 0, 2, 3]),
      StandardMaterial::new(None),
    ))
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

        ui.add(egui::Slider::new(&mut self.x, 0.0..=1.0))
          .on_hover_text("Slider");
      });
  }

  fn settings() -> FoxyCreateInfo {
    FoxyCreateInfo::default()
      .with_size(800, 600)
      .with_debug_info(DebugInfo::Shown)
      .with_polling(Polling::Poll)
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
