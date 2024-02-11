use std::sync::Arc;

use wgpu::{Device, TextureFormat};

use super::{vertex::Vertex, Renderer};

#[repr(C)]
pub struct MaterialUniforms {
  pub color: [f32; 4],
}

pub trait Material {
  fn new(device: &Device) -> Arc<Self>
  where
    Self: Sized;

  fn format() -> TextureFormat
  where
    Self: Sized,
  {
    Renderer::SWAPCHAIN_FORMAT
  }

  fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}

pub struct StandardMaterial {
  // pub uniforms: MaterialUniforms,
  // pub uniforms_buffer: wgpu::Buffer,
  pub shader: wgpu::ShaderModule,
  pub pipeline: wgpu::RenderPipeline,
}

impl Material for StandardMaterial {
  fn new(device: &Device) -> Arc<Self> {
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
        targets: &[Some(Self::format().into())],
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
    });

    Arc::new(Self { shader, pipeline })
  }

  fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
    render_pass.set_pipeline(&self.pipeline);
  }
}
