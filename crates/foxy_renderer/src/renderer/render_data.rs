use std::fmt::Debug;

#[derive(Default)]
pub struct RenderData {}

impl Debug for RenderData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "RenderData {{ .. }}")
  }
}

pub trait Drawable {
  fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}