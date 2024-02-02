use ash::vk;
use strum::EnumDiscriminants;

use super::{PipelineDiscriminants, PipelineType};
use crate::vulkan::{device::Device, error::VulkanError};

pub mod compute;

#[derive(EnumDiscriminants)]
pub enum PipelineLayout {
  Graphics { layout: vk::PipelineLayout },
  Compute { layout: vk::PipelineLayout },
}

impl PipelineLayout {
  pub fn new<P: PipelineType>(
    device: &Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
  ) -> Result<Self, VulkanError> {
    Ok(match P::kind() {
      PipelineDiscriminants::Graphics => {
        unimplemented!("graphics pipelines aren't implemented")
      }
      PipelineDiscriminants::Compute => {
        let layout_info = vk::PipelineLayoutCreateInfo {
          set_layout_count: 1,
          p_set_layouts: &descriptor_set_layout,
          ..Default::default()
        };
        let layout = unsafe { device.logical().create_pipeline_layout(&layout_info, None) }?;
        Self::Compute { layout }
      }
    })
  }

  pub fn delete(&mut self, device: &Device) {}

  pub fn layout(&self) -> vk::PipelineLayout {
    match self {
      PipelineLayout::Graphics { layout } => *layout,
      PipelineLayout::Compute { layout } => *layout,
    }
  }
}
