use std::{fmt::Debug, sync::Arc};

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, Device, IndexFormat, TextureFormat};

#[derive(Default)]
pub struct RenderData {}

impl Debug for RenderData {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "RenderData {{ .. }}")
  }
}

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

#[repr(C)]
pub struct MaterialUniforms {
  pub color: [f32; 4],
}

pub trait Material {
  fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}

pub struct StandardMaterial {
  // pub uniforms: MaterialUniforms,
  // pub uniforms_buffer: wgpu::Buffer,
  pub shader: wgpu::ShaderModule,
  pub pipeline: wgpu::RenderPipeline,
}

impl StandardMaterial {
  pub fn new(device: &Device, format: TextureFormat) -> Arc<Self> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
        "../../assets/foxy_renderer/shaders/shader.wgsl"
      ))),
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: None,
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[Vertex::desc()],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(format.into())],
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
    });

    Arc::new(Self { shader, pipeline })
  }
}

impl Material for StandardMaterial {
  fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
    render_pass.set_pipeline(&self.pipeline);
  }
}

pub struct Mesh {
  vertex_buffer: wgpu::Buffer,
  num_vertices: u32,
  index_buffer: Option<(wgpu::Buffer, u32)>,
  material: Arc<dyn Material>,
}

impl Mesh {
  pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: Option<&[u32]>, material: Arc<impl Material + 'static>) -> Self {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(vertices),
      usage: wgpu::BufferUsages::VERTEX,
    });
    let num_vertices = vertices.len() as u32;

    let index_buffer = if let Some(indices) = indices {
      let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
      });
      Some((buffer, indices.len() as u32))
    } else {
      None
    };

    Self {
      vertex_buffer,
      num_vertices,
      index_buffer,
      material,
    }
  }

  pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
    self.material.bind(render_pass);
    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    if let Some((buffer, count)) = &self.index_buffer {
      render_pass.set_index_buffer(buffer.slice(..), IndexFormat::Uint32);
      render_pass.draw_indexed(0..*count, 0, 0..1);
    } else {
      render_pass.draw(0..self.num_vertices, 0..1);
    }
  }
}
