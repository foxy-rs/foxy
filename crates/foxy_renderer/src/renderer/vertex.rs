use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};

// pub trait Vertex {
//   fn desc() -> wgpu::VertexBufferLayout<'static>;
// }

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub color: [f32; 3],
}

impl Vertex {
  pub fn desc() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &ATTRIBUTES,
    }
  }

  pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
    self.position = [x, y, z];

    self
  }

  pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
    self.color = [r, g, b];

    self
  }
}
