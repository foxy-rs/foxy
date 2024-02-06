use std::sync::Arc;

use foxy_utils::time::Time;
use tracing::debug;
use wgpu::SurfaceError;
use winit::{event::WindowEvent, window::Window};

use self::render_data::RenderData;
use crate::error::RendererError;

pub mod render_data;

// Renderer is just a thin wrapper to allow for other APIs in the future if I so
// please
pub struct Renderer {
  window: Arc<Window>,
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
}

impl Renderer {
  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    pollster::block_on(async {
      let size = window.inner_size();

      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
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
        .find(|f| *f == wgpu::TextureFormat::Rgba8UnormSrgb)
        .unwrap_or(*surface_caps.formats.first().unwrap());

      let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
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
        device,
        queue,
        config,
      })
    })
  }

  pub fn window(&self) -> &Window {
    self.window.as_ref()
  }

  pub fn resize(&mut self) {
    let new_size = self.window.inner_size();
    if new_size.width > 0 && new_size.height > 0 {
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }

  pub fn draw(&mut self, render_time: Time, render_data: Option<RenderData>) -> Result<bool, RendererError> {
    if let Some(render_data) = render_data {
      match self.surface.get_current_texture() {
        Ok(output) => {
          // render
          let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
          let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
          });
          {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
              label: Some("Render Pass"),
              color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                  load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                  }),
                  store: wgpu::StoreOp::Store,
                },
              })],
              depth_stencil_attachment: None,
              occlusion_query_set: None,
              timestamp_writes: None,
            });
          }


          // submit will accept anything that implements IntoIter
          self.queue.submit(std::iter::once(encoder.finish()));
          
          self.window.pre_present_notify();
          output.present();

          Ok(true)
        }
        Err(SurfaceError::Lost) => {
          self.resize();
          Ok(false)
        }
        Err(error) => Err(error)?,
      }
    } else {
      Ok(false)
    }
  }
}
