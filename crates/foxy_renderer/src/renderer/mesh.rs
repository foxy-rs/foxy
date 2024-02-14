use std::sync::Arc;

use wgpu::{util::DeviceExt, IndexFormat};

use super::{material::Material, render_data::Drawable, vertex::Vertex};

pub struct VertexData {
  buffer: wgpu::Buffer,
  count: u32,
}

pub struct IndexData {
  buffer: wgpu::Buffer,
  count: u32,
}

pub struct Mesh {
  pub vertices: VertexData,
  pub indices: Option<IndexData>,
  pub material: Arc<dyn Material>,
}

impl Mesh {
  pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: Option<&[u32]>, material: Arc<dyn Material>) -> Self {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(vertices),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let vertices = VertexData {
      buffer: vertex_buffer,
      count: vertices.len() as u32,
    };

    let indices = if let Some(indices) = indices {
      let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
      });

      Some(IndexData {
        buffer: index_buffer,
        count: indices.len() as u32,
      })
    } else {
      None
    };

    Self {
      vertices,
      indices,
      material,
    }
  }
}

impl Drawable for Mesh {
  fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
    render_pass.set_vertex_buffer(0, self.vertices.buffer.slice(..));
    if let Some(indices) = &self.indices {
      render_pass.set_index_buffer(indices.buffer.slice(..), IndexFormat::Uint32);
      render_pass.draw_indexed(0..indices.count, 0, 0..1);
    } else {
      render_pass.draw(0..self.vertices.count, 0..1);
    }
  }
}
