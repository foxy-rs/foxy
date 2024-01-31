use ash::vk;
use foxy_utils::types::handle::Handle;

use super::{device::Device, error::VulkanError, instance::Instance, surface::Surface};
use crate::vulkan_error;

#[derive(Debug, Default)]
pub struct FrameData {
  pub command_pool: vk::CommandPool,
  pub master_command_buffer: vk::CommandBuffer,
  render_fence: vk::Fence,
  swapchain_semaphore: vk::Semaphore,
  render_semaphore: vk::Semaphore,
}

impl FrameData {
  pub const FRAME_OVERLAP: usize = 2;

  pub fn new(surface: &Surface, instance: &Instance, device: &Device) -> Result<FrameData, VulkanError> {
    let create_info = vk::CommandPoolCreateInfo::default()
      .queue_family_index(device.graphics().family())
      .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    let command_pool = unsafe { device.logical().create_command_pool(&create_info, None) }?;

    let buffer_info = vk::CommandBufferAllocateInfo::default()
      .command_pool(command_pool)
      .command_buffer_count(1)
      .level(vk::CommandBufferLevel::PRIMARY);

    let master_command_buffer = unsafe { device.logical().allocate_command_buffers(&buffer_info) }?
      .first()
      .cloned()
      .ok_or_else(|| vulkan_error!("invalid command buffers size"))?;

    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    let render_fence = unsafe { device.logical().create_fence(&fence_info, None) }?;

    let semaphore_info = vk::SemaphoreCreateInfo::default();
    let swapchain_semaphore = unsafe { device.logical().create_semaphore(&semaphore_info, None) }?;
    let render_semaphore = unsafe { device.logical().create_semaphore(&semaphore_info, None) }?;

    Ok(FrameData {
      command_pool,
      master_command_buffer,
      render_fence,
      swapchain_semaphore,
      render_semaphore,
    })
  }

  pub fn delete(&mut self, device: &mut Handle<Device>) {
    unsafe {
      device.get_mut().logical().destroy_command_pool(self.command_pool, None);
      device.get_mut().logical().destroy_fence(self.render_fence, None);
      device.get_mut().logical().destroy_semaphore(self.swapchain_semaphore, None);
      device.get_mut().logical().destroy_semaphore(self.render_semaphore, None);
    }
  }
}
