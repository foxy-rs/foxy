use std::sync::Arc;

use foxy_utils::time::Time;
use winit::{event::WindowEvent, window::Window};

use self::render_data::RenderData;
use crate::{error::RendererError, vulkan::Vulkan};

pub mod command;
pub mod render_data;

// Renderer is just a thin wrapper to allow for other APIs in the future if I so
// please
pub struct Renderer {
  backend: Vulkan,
}

impl Renderer {
  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    let backend = Vulkan::new(window)?;

    Ok(Self { backend })
  }

  pub fn delete(&mut self) {
    self.backend.delete();
  }

  pub fn render(&mut self, render_time: Time, render_data: RenderData) -> Result<(), RendererError> {
    self.backend.render(render_time, render_data)?;
    Ok(())
  }

  pub fn resize(&mut self) {}

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }
}
