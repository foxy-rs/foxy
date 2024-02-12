use std::sync::Arc;

use tracing::debug;
use wgpu::TextureFormat;
use winit::window::Window;

use crate::error::RendererError;

pub struct GraphicsContext {
  window: Arc<Window>,
  surface: wgpu::Surface<'static>,
  config: wgpu::SurfaceConfiguration,
  device: wgpu::Device,
  queue: wgpu::Queue,
}

impl GraphicsContext {
  pub const SURFACE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    pollster::block_on(async {
      let size = window.inner_size();

      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
      });

      let surface = instance.create_surface(window.clone())?;

      let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::HighPerformance,
          compatible_surface: Some(&surface),
          force_fallback_adapter: false,
        })
        .await
        .expect("failed to request adapter");

      let (device, queue) = adapter
        .request_device(
          &wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
          },
          None,
        )
        .await?;

      let surface_caps = surface.get_capabilities(&adapter);
      debug!("{surface_caps:#?}");
      let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| *f == Self::SURFACE_FORMAT)
        .unwrap_or(*surface_caps.formats.first().unwrap());

      let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        alpha_mode: *surface_caps.alpha_modes.first().unwrap(),
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
      };

      surface.configure(&device, &config);

      Ok(Self {
        window,
        surface,
        config,
        device,
        queue,
      })
    })
  }

  pub fn device(&self) -> &wgpu::Device {
    &self.device
  }

  pub fn queue(&self) -> &wgpu::Queue {
    &self.queue
  }

  pub fn surface(&self) -> &wgpu::Surface<'static> {
    &self.surface
  }

  pub fn reconfigure(&mut self) {
    let new_size = self.window.inner_size();
    self.config.width = new_size.width.max(1);
    self.config.height = new_size.height.max(1);
    self.surface.configure(&self.device, &self.config);
  }

  pub fn config(&self) -> &wgpu::SurfaceConfiguration {
    &self.config
  }
}
