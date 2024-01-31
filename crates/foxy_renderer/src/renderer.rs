use foxy_utils::{time::Time, types::primitives::Dimensions};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use self::render_data::RenderData;
use crate::error::RendererError;

pub mod command;
pub mod render_data;

pub trait RenderBackend {
  fn new(window: impl HasRawDisplayHandle + HasRawWindowHandle, window_size: Dimensions) -> Result<Self, RendererError>
  where
    Self: Sized;

  fn delete(&mut self);

  fn draw(&mut self, render_time: Time) -> Result<(), RendererError>;
}

// Renderer is just a thin wrapper to allow for other APIs in the future if I so
// please
pub struct Renderer<B: RenderBackend> {
  backend: B,
  render_data: RenderData,
}

impl<B: RenderBackend> Renderer<B> {
  pub fn new(
    window: impl HasRawDisplayHandle + HasRawWindowHandle,
    window_size: Dimensions,
  ) -> Result<Self, RendererError> {
    let backend = B::new(window, window_size)?;

    Ok(Self {
      backend,
      render_data: RenderData::default(),
    })
  }

  pub fn delete(&mut self) {
    self.backend.delete();
  }

  pub fn draw_frame(&mut self, render_time: Time) -> Result<(), RendererError> {
    self.backend.draw(render_time)
  }

  pub fn update_render_data(&mut self, render_data: RenderData) {
    self.render_data = render_data;
  }
}
