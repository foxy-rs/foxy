use std::sync::Arc;

use foxy_utils::time::Time;
use tracing::debug;
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use self::render_data::RenderData;
use crate::{error::RendererError, renderer::render_data::VERTICES};

pub mod render_data;

pub struct Renderer {
  window: Arc<Window>,
  surface: wgpu::Surface<'static>,
  config: wgpu::SurfaceConfiguration,
  device: wgpu::Device,
  queue: wgpu::Queue,
  target_format: wgpu::TextureFormat,
  render_target: wgpu::Texture,

  shader: wgpu::ShaderModule,
  pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,

  is_dirty: bool,
}

impl Renderer {
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

      let target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
      let render_target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Render Target"),
        size: wgpu::Extent3d {
          width: size.width,
          height: size.height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
      });

      let surface_caps = surface.get_capabilities(&adapter);
      debug!("{surface_caps:#?}");
      let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| *f == wgpu::TextureFormat::Rgba8UnormSrgb)
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

      let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
          "../assets/foxy_renderer/shaders/shader.wgsl"
        ))),
      });

      let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
          module: &shader,
          entry_point: "vs_main",
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &shader,
          entry_point: "fs_main",
          targets: &[Some(target_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
      });

      let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
      });

      Ok(Self {
        window,
        surface,
        config,
        device,
        queue,
        target_format,
        render_target,
        shader,
        pipeline,
        vertex_buffer,
        is_dirty: false,
      })
    })
  }

  pub fn window(&self) -> &Window {
    self.window.as_ref()
  }

  pub fn refresh(&mut self) {
    self.is_dirty = true;
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }

  pub fn draw(&mut self, render_time: Time, render_data: RenderData) -> Result<(), RendererError> {
    match self.next_frame() {
      Ok(frame) => {
        let view = self.render_target.create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("Render Encoder"),
        });

        {
          let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
              view: &view,
              resolve_target: None,
              ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                store: wgpu::StoreOp::Store,
              },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
          });

          render_pass.set_pipeline(&self.pipeline);
          render_pass.draw(0..3, 0..1);
        }

        command_encoder.copy_texture_to_texture(
          self.render_target.as_image_copy(),
          frame.texture.as_image_copy(),
          wgpu::Extent3d {
            width: self.window.inner_size().width,
            height: self.window.inner_size().height,
            depth_or_array_layers: 1,
          },
        );

        // submit will accept anything that implements IntoIter
        self.queue.submit(Some(command_encoder.finish()));

        self.window.pre_present_notify();
        frame.present();

        Ok(())
      }
      Err(RendererError::RebuildSwapchain) => Ok(()),
      Err(error) => Err(error),
    }
  }
}

impl Renderer {
  fn rebuild_swapchain(&mut self) {
    let new_size = self.window.inner_size();
    self.config.width = new_size.width.max(1);
    self.config.height = new_size.height.max(1);
    self.surface.configure(&self.device, &self.config);
    //
    // self.render_target.destroy();
    self.render_target = self.device.create_texture(&wgpu::TextureDescriptor {
      label: Some("Render Target"),
      size: wgpu::Extent3d {
        width: self.window().inner_size().width,
        height: self.window().inner_size().height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: self.target_format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });
  }

  fn next_frame(&mut self) -> Result<wgpu::SurfaceTexture, RendererError> {
    if self.is_dirty {
      self.rebuild_swapchain();
      self.is_dirty = false;
    }

    match self.surface.get_current_texture() {
      Ok(frame) => {
        if frame.suboptimal {
          self.refresh();
        }
        Ok(frame)
      }
      Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated | wgpu::SurfaceError::OutOfMemory) => {
        self.refresh();
        Err(RendererError::RebuildSwapchain)
      }
      Err(error) => Err(RendererError::SurfaceError(error)),
    }
  }
}
