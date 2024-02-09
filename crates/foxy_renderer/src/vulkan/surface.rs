use std::sync::Arc;

use vulkano::{instance::Instance, swapchain::Surface};
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
