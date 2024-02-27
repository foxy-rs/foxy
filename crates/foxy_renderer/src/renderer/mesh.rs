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

#[derive(Clone)]
pub struct StaticMesh {
  pub vertices: Vec<Vertex>,
  pub indices: Option<Vec<u32>>,
  pub material: Arc<dyn Material + Send + Sync>,
}

impl StaticMesh {
  pub fn new(vertices: &[Vertex], indices: Option<&[u32]>, material: Arc<dyn Material + Send + Sync>) -> Self {
    Self {
      vertices: vertices.to_vec(),
      indices: indices.map(|i| i.to_vec()),
      material,
    }
  }

  pub fn bake(&self, device: &wgpu::Device) -> BakedStaticMesh {
    BakedStaticMesh::new(device, self)
  }
}

pub struct BakedStaticMesh {
  pub vertices: VertexData,
  pub indices: Option<IndexData>,
  pub material: Arc<dyn Material + Send + Sync>,
}

impl BakedStaticMesh {
  pub fn new(device: &wgpu::Device, mesh: &StaticMesh) -> Self {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&mesh.vertices),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let vertices = VertexData {
      buffer: vertex_buffer,
      count: mesh.vertices.len() as u32,
    };

    let indices = if let Some(indices) = &mesh.indices {
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
      material: mesh.material.clone(),
    }
  }
}

impl Drawable for BakedStaticMesh {
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
