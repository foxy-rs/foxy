use std::sync::Arc;

use ash::vk;

use crate::{device::Device, error::VulkanError};

#[derive(Clone)]
pub struct PipelineLayout {
  layout: vk::PipelineLayout,
}

impl Drop for PipelineLayout {
  fn drop(&mut self) {
    unsafe {}
  }
}

impl Default for PipelineLayout {
  fn default() -> Self {
    Self {
      layout: vk::PipelineLayout::null(),
    }
  }
}

impl PipelineLayout {
  pub fn new(device: Arc<Device>) -> Result<Self, VulkanError> {
    todo!()
  }

  pub fn layout(&self) -> vk::PipelineLayout {
    self.layout
  }
}
