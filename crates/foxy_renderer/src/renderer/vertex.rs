use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};

// pub trait Vertex {
//   fn desc() -> wgpu::VertexBufferLayout<'static>;
// }

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
  pub position: [f32; 3],
  pub color: [f32; 4],
  pub uv: [f32; 2],
}

impl Default for Vertex {
  fn default() -> Self {
    Self {
      position: [0., 0., 0.],
      color: [1., 1., 1., 1.],
      uv: [0., 0.],
    }
  }
}

impl Vertex {
  pub fn desc() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
      wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2];
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

  pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
    self.color = [r, g, b, a];
    self
  }

  pub fn with_uvs(mut self, u: f32, v: f32) -> Self {
    self.uv = [u, v];
    self
  }
}
