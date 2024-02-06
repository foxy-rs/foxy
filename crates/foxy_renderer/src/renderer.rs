use foxy_utils::time::Time;
use winit::{dpi::PhysicalSize, window::Window};

use self::render_data::RenderData;
use crate::error::RendererError;

pub mod command;
pub mod render_data;

// Renderer is just a thin wrapper to allow for other APIs in the future if I so
// please
pub struct Renderer {
  
}

impl Renderer {
  pub fn new(window: &Window, size: PhysicalSize<u32>) -> Result<Self, RendererError> {
    
    Ok(Self { })
  }

  pub fn delete(&mut self) {
    
  }

  pub fn draw(&mut self, render_time: Time, render_data: Option<RenderData>) -> Result<bool, RendererError> {
    if let Some(render_data) = render_data {
      
      Ok(true)
    } else {
      Ok(false)
    }
  }
}
