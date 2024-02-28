use uuid::Uuid;
use wgpu::CommandEncoder;

use super::Pass;
use crate::renderer::{
  asset_manager::{AssetManager, RenderPipelineInfo},
  diffuse_texture::DiffuseTexture,
  mesh::BakedStaticMesh,
  render_data::Drawable,
  target::RenderTarget,
  vertex::Vertex,
  Renderer,
};

pub struct SimplePass {
  pipeline_layout: wgpu::PipelineLayout,
}

impl SimplePass {
  pub fn new(device: &wgpu::Device) -> Self {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Simple Pipeline Layout"),
      bind_group_layouts: &[DiffuseTexture::bind_group_layout(device)],
      push_constant_ranges: &[],
    });

    Self { pipeline_layout }
  }
}

impl Pass for SimplePass {
  fn draw(
    &mut self,
    command_encoder: &mut CommandEncoder,
    asset_manager: &AssetManager,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view: &wgpu::TextureView,
    mesh: Option<&BakedStaticMesh>,
  ) -> Result<(), crate::error::RendererError> {
    let Some(mesh) = mesh else { return Ok(()) };

    let shader = asset_manager.read_shader(mesh.material.shader(), device);
    let texture = asset_manager.read_texture(mesh.material.albedo(), device, queue);

    let pipeline = asset_manager.create_render_pipeline(device, &RenderPipelineInfo {
      id: Uuid::from_u128(0xe867c7ec8ca44b6ebe9a9281b94051ac),
      label: Some("Simple Pipeline"),
      layout: &self.pipeline_layout,
      color_format: Renderer::RENDER_TARGET_FORMAT,
      depth_format: None,
      vertex_layouts: &[Vertex::desc()],
      shader: &shader,
    });

    let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("Simple Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      occlusion_query_set: None,
      timestamp_writes: None,
    });

    render_pass.set_pipeline(&pipeline);
    render_pass.set_bind_group(0, &texture.bind_group, &[]);
    mesh.draw(&mut render_pass);

    Ok(())
  }

  fn resize(&mut self, _device: &wgpu::Device, _render_target: &RenderTarget) {}
}
