use ash::vk;
use foxy_types::handle::Handle;
use tracing::debug;

use crate::{device::Device, error::VulkanError};

#[derive(Clone)]
pub struct PipelineLayout {
  device: Handle<Device>,
  layout: vk::PipelineLayout,
}

impl PipelineLayout {
  pub fn new(device: Handle<Device>) -> Result<Self, VulkanError> {
    debug!("new pipeline layout!");
    let layout_info = vk::PipelineLayoutCreateInfo::default();
    let layout = unsafe { device.get().logical().create_pipeline_layout(&layout_info, None) }?;
    Ok(Self { device, layout })
  }

  pub fn delete(&mut self) {
    debug!("delete pipeline layout!");
    unsafe {
      self.device.get().logical().destroy_pipeline_layout(self.layout, None);
    }
  }

  pub fn layout(&self) -> vk::PipelineLayout {
    self.layout
  }
}
