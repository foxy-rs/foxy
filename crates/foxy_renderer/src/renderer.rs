use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use foxy_utils::time::Time;
use itertools::Itertools;
use wgpu::{Color, TextureFormat};
use winit::window::Window;

use self::{
  context::GraphicsContext,
  render_data::RenderData,
  render_pass::{simple::SimplePass, tonemap::ToneMapPass, Pass},
  target::RenderTarget,
};
use crate::{egui::EguiRenderer, error::RendererError, renderer::asset_manager::AssetManager};

pub mod asset_manager;
pub mod context;
pub mod material;
pub mod mesh;
pub mod render_data;
pub mod render_pass;
pub mod shader;
pub mod target;
pub mod texture;
pub mod vertex;

pub struct Renderer {
  window: Arc<Window>,
  context: GraphicsContext,
  egui: EguiRenderer,
  render_target: RenderTarget,

  asset_manager: AssetManager,

  simple_pass: SimplePass,
  tone_map_pass: ToneMapPass,

  is_dirty: bool,
}

impl Renderer {
  const CLEAR_VALUE: Color = Color {
    r: 0.1,
    g: 0.4,
    b: 1.0,
    a: 1.0,
  };
  pub const RENDER_TARGET_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
  pub const SURFACE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

  pub fn new(window: Arc<Window>, egui_context: egui::Context) -> Result<Self, RendererError> {
    pollster::block_on(async {
      let context = GraphicsContext::new(window.clone())?;
      let egui = EguiRenderer::new(window.clone(), context.device(), egui_context, Self::SURFACE_FORMAT, None, 1);

      let render_target = RenderTarget::new(window.clone(), context.device());

      let asset_manager = AssetManager::new();

      let simple_pass = SimplePass::new(context.device());
      let tone_map_pass = ToneMapPass::new(context.device(), &render_target);

      Ok(Self {
        window,
        context,
        egui,
        render_target,
        asset_manager,
        simple_pass,
        tone_map_pass,
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

  pub fn render_frame(&mut self, _render_time: Time, render_data: RenderData) -> Result<(), RendererError> {
    match self.next_frame() {
      Ok(frame) => {
        let swapchain_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder = self
          .context
          .device()
          .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
          });

        {
          // clear attachment
          let _render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clearing Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
              view: &self.render_target.view,
              resolve_target: None,
              ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(Self::CLEAR_VALUE),
                store: wgpu::StoreOp::Store,
              },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
          });
        }

        let meshes = render_data
          .meshes
          .iter()
          .map(|m| m.bake(self.context.device()))
          .collect_vec();

        for mesh in meshes.iter() {
          self.simple_pass.draw(
            &mut command_encoder,
            &self.asset_manager,
            self.context.device(),
            self.context.queue(),
            &self.render_target.view,
            Some(mesh),
          )?;
        }

        // Finish by rendering onto the primary view
        self.tone_map_pass.draw(
          &mut command_encoder,
          &self.asset_manager,
          self.context.device(),
          self.context.queue(),
          &swapchain_view,
          None,
        )?;

        // EGUI

        let screen_descriptor = ScreenDescriptor {
          size_in_pixels: [self.context.config().width, self.context.config().height],
          pixels_per_point: self.window().scale_factor() as f32,
        };

        self.egui.draw(
          self.context.device(),
          self.context.queue(),
          &mut command_encoder,
          &swapchain_view,
          screen_descriptor,
          render_data.full_output,
        );

        // submit will accept anything that implements IntoIter
        self.context.queue().submit(Some(command_encoder.finish()));
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
  fn reconfigure(&mut self) {
    self.context.reconfigure();
    self.render_target.resize(self.context.device());
    self.simple_pass.resize(self.context.device(), &self.render_target);
    self.tone_map_pass.resize(self.context.device(), &self.render_target);
  }

  fn next_frame(&mut self) -> Result<wgpu::SurfaceTexture, RendererError> {
    if self.is_dirty {
      self.reconfigure();
      self.is_dirty = false;
    }

    match self.context.surface().get_current_texture() {
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
