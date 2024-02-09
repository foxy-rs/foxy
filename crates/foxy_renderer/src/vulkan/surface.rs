use std::sync::Arc;

use vulkano::{
  format::Format,
  instance::Instance,
  swapchain::{PresentMode, Surface, SurfaceCapabilities},
};
use winit::window::Window;

use super::error::VulkanError;

pub struct FoxySurface {
  surface: Arc<Surface>,
}

impl FoxySurface {
  pub fn new(instance: Arc<Instance>, window: Arc<Window>) -> Result<Self, VulkanError> {
    let surface = Surface::from_window(instance, window)?;
    Ok(Self { surface })
  }

  pub fn vk(&self) -> &Arc<Surface> {
    &self.surface
  }
}

pub struct SwapchainSupport {
  pub capabilities: SurfaceCapabilities,
  pub formats: Vec<Format>,
  pub present_modes: Vec<PresentMode>,
}
