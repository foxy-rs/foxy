// use ash::vk;

// use super::{
//   layout::{compute::ComputePipelineLayout, PipelineLayout},
//   Pipeline,
// };
// use crate::{
//   vulkan::{device::Device, error::VulkanError, shader::set::HasCompute},
//   vulkan_error,
// };

// pub struct ComputePipeline {
//   device: Device,
//   pipeline: vk::Pipeline,
//   layout: vk::PipelineLayout,
// }

// impl Pipeline<HasCompute> for ComputePipeline {
//   type Compute = HasCompute;

//   fn new(
//     device: Device,
//     shader_set: Self::ShaderSet,
//     layout: impl PipelineLayout<Self::Compute>,
//   ) -> Result<Self, VulkanError>
//   where
//     Self: Sized,
//   {
//     let shader_info = shader_set.compute().pipeline_info();
//     let layout = ComputePipelineLayout::new(&device, descriptor_set_layout)?.layout();

//     let pipeline_create_info = vk::ComputePipelineCreateInfo::builder()
//       .stage(shader_info)
//       .layout(layout);

//     let pipeline = unsafe {
//       device
//         .logical()
//         .create_compute_pipelines(vk::PipelineCache::null(), &[*pipeline_create_info], None)
//         .map(|pipelines| pipelines.first().cloned())
//         .map_err(|e| e.1)
//     }?
//     .ok_or_else(|| vulkan_error!("invalid pipeline index"))?;

//     Ok(Self {
//       device,
//       pipeline,
//       layout,
//     })
//   }

//   fn delete(&mut self) {
//     unsafe {
//       self.device.logical().destroy_pipeline(self.pipeline, None);
//     }
//   }

//   fn bind(&self, command_buffer: vk::CommandBuffer) {
//     unsafe {
//       self
//         .device
//         .logical()
//         .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline)
//     };
//   }
// }
