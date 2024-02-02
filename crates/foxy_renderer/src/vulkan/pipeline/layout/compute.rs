// use ash::vk;

// use super::PipelineLayout;
// use crate::vulkan::{device::Device, error::VulkanError, shader::set::HasCompute};

// #[derive(Clone)]
// pub struct ComputePipelineLayout {
//   layout: vk::PipelineLayout,
// }

// impl PipelineLayout<HasCompute> for ComputePipelineLayout {
//   fn new(device: &Device, descriptor_set_layout: vk::DescriptorSetLayout) -> Result<Self, VulkanError> {
//     let layout_info = vk::PipelineLayoutCreateInfo {
//       set_layout_count: 1,
//       p_set_layouts: &descriptor_set_layout,
//       ..Default::default()
//     };

//     let layout = unsafe { device.logical().create_pipeline_layout(&layout_info, None) }?;
//     Ok(Self { layout })
//   }

//   fn delete(&mut self, device: &Device) {
//     unsafe {
//       device.logical().destroy_pipeline_layout(self.layout, None);
//     }
//   }

//   fn layout(&self) -> vk::PipelineLayout {
//     self.layout
//   }
// }
