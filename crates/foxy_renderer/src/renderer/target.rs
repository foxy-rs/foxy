use std::sync::Arc;

use wgpu::Device;
use winit::window::Window;

use super::Renderer;

pub struct RenderTarget {
  window: Arc<Window>,
  pub texture: wgpu::Texture,
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
}

impl RenderTarget {
  pub fn new(window: Arc<Window>, device: &Device) -> Self {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("HDR Render Texture"),
      size: wgpu::Extent3d {
        width: window.inner_size().width,
        height: window.inner_size().height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Renderer::RENDER_TARGET_FORMAT,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("HDR Render Sampler"),
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

    Self {
      window,
      texture,
      view,
      sampler,
    }
  }

  pub fn resize(&mut self, device: &Device) {
    *self = Self::new(self.window.clone(), device);
  }
}
