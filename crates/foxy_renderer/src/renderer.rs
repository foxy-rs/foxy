use foxy_utils::{time::Time, types::handle::Handle};
use foxy_window::window::Window;

use crate::error::RendererError;

pub mod command;
pub mod render_data;

pub trait RenderBackend {
  fn new(window: Handle<Window>) -> Result<Self, RendererError>
  where
    Self: Sized;

  fn delete(&mut self);

  fn draw(&mut self, render_time: Time) -> Result<(), RendererError>;
}

// Renderer is just a thin wrapper to allow for other APIs in the future if I so
// please
pub struct Renderer<B: RenderBackend> {
  backend: B,
}

impl<B: RenderBackend> Renderer<B> {
  pub fn new(window: Handle<Window>) -> Result<Self, RendererError> {
    let backend = B::new(window)?;

    Ok(Self { backend })
  }

  pub fn delete(&mut self) {
    self.backend.delete();
  }

  pub fn draw_frame(&mut self, render_time: Time) -> Result<(), RendererError> {
    self.backend.draw(render_time)
  }
}
