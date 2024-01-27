use crate::error::VulkanError;


pub struct Swapchain {}

impl Swapchain {
  const MAX_FRAMES_IN_FLIGHT: u32 = 2;

  pub fn new() -> Result<Self, VulkanError> {
    Ok(Self {})
  }
}

impl Drop for Swapchain {
  fn drop(&mut self) {
    unsafe {
      
    }
  }
}