use std::fmt::Debug;

use egui::FullOutput;

#[derive(Default)]
pub struct RenderData {
  pub full_output: FullOutput
}

impl Debug for RenderData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "RenderData {{ .. }}")
  }
}

pub trait Drawable {
  fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}