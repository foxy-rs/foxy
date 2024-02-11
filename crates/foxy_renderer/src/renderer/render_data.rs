use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};

#[derive(Default)]
pub struct RenderData {}

impl Debug for RenderData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "RenderData {{ .. }}")
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
  position: [f32; 3],
  color: [f32; 3],
}

pub const VERTICES: &[Vertex] = &[
  Vertex {
    position: [0.0, 0.5, 0.0],
    color: [1.0, 0.0, 0.0],
  },
  Vertex {
    position: [-0.5, -0.5, 0.0],
    color: [0.0, 1.0, 0.0],
  },
  Vertex {
    position: [0.5, -0.5, 0.0],
    color: [0.0, 0.0, 1.0],
  },
];
