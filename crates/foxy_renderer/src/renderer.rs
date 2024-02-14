use std::sync::Arc;

use foxy_utils::time::Time;
use image::{DynamicImage, GenericImageView};
use tracing::debug;
use wgpu::{Color, TextureFormat};
use winit::{event::WindowEvent, window::Window};

use self::{
  context::GraphicsContext,
  material::StandardMaterial,
  mesh::Mesh,
  render_data::{Drawable, RenderData},
  render_pass::{simple::SimplePass, tonemap::ToneMapPass, Pass},
  target::RenderTarget,
};
use crate::{
  error::RendererError,
  renderer::{material::Material, texture::DiffuseTexture, vertex::Vertex},
};

mod context;
// mod hdr_pipeline;
pub mod material;
pub mod mesh;
pub mod render_data;
pub mod render_pass;
mod target;
pub mod texture;
pub mod vertex;

pub struct Renderer {
  window: Arc<Window>,
  context: GraphicsContext,
  render_target: RenderTarget,

  simple_pass: SimplePass,
  tone_map_pass: ToneMapPass,

  textured_material: Arc<StandardMaterial>,
  standard_material: Arc<StandardMaterial>,
  mesh: Mesh,

  is_dirty: bool,
}

impl Renderer {
  const CLEAR_VALUE: Color = Color {
    r: 0.1,
    g: 0.4,
    b: 1.0,
    a: 1.0,
  };

  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    pollster::block_on(async {
      let context = GraphicsContext::new(window.clone())?;
      let render_target = RenderTarget::new(window.clone(), context.device());

      let simple_pass = SimplePass::new(context.device());
      let tone_map_pass = ToneMapPass::new(context.device(), context.config(), &render_target);

      let diffuse_texture = DiffuseTexture::new(
        context.device(),
        context.queue(),
        include_bytes!("../assets/textures/cobblestone.png"),
      );

      let textured_material = StandardMaterial::new(context.device(), context.queue(), Some(diffuse_texture));
      let standard_material = StandardMaterial::new(context.device(), context.queue(), None);

      let mesh = Mesh::new(
        context.device(),
        &[
          Vertex {
            position: [-0.5, -0.5, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
            uv: [0., 1.],
            ..Default::default()
          },
          Vertex {
            position: [0.5, -0.5, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
            uv: [1., 1.],
            ..Default::default()
          },
          Vertex {
            position: [0.5, 0.5, 0.0],
            color: [0.0, 1.0, 0.0, 1.0],
            uv: [1., 0.],
            ..Default::default()
          },
          Vertex {
            position: [-0.5, 0.5, 0.0],
            color: [0.0, 0.0, 1.0, 1.0],
            uv: [0., 0.],
            ..Default::default()
          },
        ],
        Some(&[0, 1, 2, 0, 2, 3]),
        textured_material.clone(),
      );

      // let mesh = Mesh::new(
      //   context.device(),
      //   &[
      //     Vertex {
      //       position: [-0.5, -0.5, 0.0],
      //       color: [1.0, 0.0, 0.0, 1.0],
      //       uv: [1., 0.],
      //       ..Default::default()
      //     },
      //     Vertex {
      //       position: [0.5, -0.5, 0.0],
      //       color: [0.0, 1.0, 0.0, 1.0],
      //       uv: [1., 1.],
      //       ..Default::default()
      //     },
      //     Vertex {
      //       position: [0.0, 0.5, 0.0],
      //       color: [0.0, 0.0, 1.0, 1.0],
      //       uv: [0., 0.],
      //       ..Default::default()
      //     },
      //   ],
      //   Some(&[0, 1, 2]),
      //   standard_material.clone(),
      // );

      Ok(Self {
        window,
        context,
        render_target,
        simple_pass,
        tone_map_pass,
        textured_material,
        standard_material,
        mesh,
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
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

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

        self
          .simple_pass
          .draw(&mut command_encoder, &self.render_target.view, &self.mesh)?;

        // Finish by rendering onto the primary view
        self.tone_map_pass.draw(&mut command_encoder, &view, &self.mesh)?;

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
