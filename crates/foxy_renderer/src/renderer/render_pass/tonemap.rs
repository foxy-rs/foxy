use wgpu::{Color, CommandEncoder};

use super::{create_render_pipeline, Pass};
use crate::renderer::{context::GraphicsContext, render_data::Drawable, target::RenderTarget, Renderer};

pub struct ToneMapPass {
  pipeline: wgpu::RenderPipeline,
  bind_group: wgpu::BindGroup,
  layout: wgpu::BindGroupLayout,
}

impl ToneMapPass {
  pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, render_target: &RenderTarget) -> Self {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
      layout: &layout,
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
      bind_group_layouts: &[&layout],
      push_constant_ranges: &[],
    });

    let shader = wgpu::include_wgsl!("../../../assets/shaders/hdr.wgsl");

    let pipeline = create_render_pipeline(
      Some("HDR Pipeline"),
      device,
      &pipeline_layout,
      config.format,
      None,
      &[],
      shader,
    );

    Self {
      pipeline,
      bind_group,
      layout,
    }
  }
}

impl Pass for ToneMapPass {
  fn draw(
    &mut self,
    command_encoder: &mut CommandEncoder,
    render_target: &wgpu::TextureView,
    mesh: &crate::renderer::mesh::Mesh,
  ) -> Result<(), crate::error::RendererError> {
    let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("HDR Pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: render_target,
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

    render_pass.set_pipeline(&self.pipeline);
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
