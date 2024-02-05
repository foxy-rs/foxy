use foxy_utils::time::Time;
use winit::{
  dpi::{LogicalSize, PhysicalSize},
  window::Window,
};

use self::render_data::RenderData;
use crate::error::RendererError;

pub mod command;
pub mod render_data;

pub trait RenderBackend {
  fn new(window: &Window, size: PhysicalSize<u32>) -> Result<Self, RendererError>
  where
    Self: Sized;

  fn delete(&mut self);

  fn draw(&mut self, render_time: Time, render_data: RenderData) -> Result<(), RendererError>;
}

// Renderer is just a thin wrapper to allow for other APIs in the future if I so
// please
pub struct Renderer<B: RenderBackend> {
  backend: B,
}

impl<B: RenderBackend> Renderer<B> {
  pub fn new(window: &Window, size: PhysicalSize<u32>) -> Result<Self, RendererError> {
    let backend = B::new(window, size)?;

    Ok(Self { backend })
  }

  pub fn delete(&mut self) {
    self.backend.delete();
  }

  pub fn draw(&mut self, render_time: Time, render_data: Option<RenderData>) -> Result<(), RendererError> {
    if let Some(render_data) = render_data {
      self.backend.draw(render_time, render_data)
    } else {
      Ok(())
    }
  }
}
