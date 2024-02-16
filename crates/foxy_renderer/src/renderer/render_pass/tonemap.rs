use uuid::Uuid;
use wgpu::CommandEncoder;

use super::Pass;
use crate::renderer::{
  asset_manager::AssetManager, mesh::BakedStaticMesh, shader::ShaderHandle, target::RenderTarget, vertex::Vertex, Renderer
};

// create from material on first frame, then cache it. need hardcoded uuids
pub struct ToneMapPass {
  pipeline_layout: wgpu::PipelineLayout,
  bind_group: wgpu::BindGroup,
  layout: wgpu::BindGroupLayout,
}

impl ToneMapPass {
  pub fn new(
    device: &wgpu::Device,
    // asset_manager: &AssetManager,
    // config: &wgpu::SurfaceConfiguration,
    render_target: &RenderTarget,
  ) -> Self {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("HDR Layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            // The Rgba16Float format cannot be filtered
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("HDR Bind Group"),
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&render_target.view),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&render_target.sampler),
        },
      ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("HDR Pipeline Layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    // let shader = wgpu::include_wgsl!("../../../assets/foxy/shaders/hdr.wgsl");

    // let pipeline =
    //   create_render_pipeline(Some("HDR Pipeline"), device, &pipeline_layout,
    // config.format, None, &[], shader);

    Self {
      pipeline_layout,
      bind_group,
      layout: bind_group_layout,
    }
  }
}

impl Pass for ToneMapPass {
  fn draw(
    &mut self,
    command_encoder: &mut CommandEncoder,
    asset_manager: &AssetManager,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    view: &wgpu::TextureView,
    _mesh: &BakedStaticMesh,
  ) -> Result<(), crate::error::RendererError> {
    let shader = asset_manager.read_shader(ShaderHandle("assets/foxy/shaders/hdr.wgsl".into()), device);
    // let texture = asset_manager.read_texture(mesh.material.albedo(), device, queue);

    let pipeline = asset_manager.create_render_pipeline(
      Uuid::from_u128(0xa6a61819d926432987cb4c7c9c665b02),
      Some("HDR Pipeline"),
      device,
      &self.pipeline_layout,
      Renderer::SURFACE_FORMAT,
      None,
      &[],
      &shader,
    );

    let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("HDR Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(Renderer::CLEAR_VALUE),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      occlusion_query_set: None,
      timestamp_writes: None,
    });

    render_pass.set_pipeline(&pipeline);
    render_pass.set_bind_group(0, &self.bind_group, &[]);

    render_pass.draw(0..3, 0..1);

    // mesh.draw(&mut render_pass);

    Ok(())
  }

  fn resize(&mut self, device: &wgpu::Device, render_target: &RenderTarget) {
    self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("HDR Bind Group"),
      layout: &self.layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(&render_target.view),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(&render_target.sampler),
        },
      ],
    });
  }
}
