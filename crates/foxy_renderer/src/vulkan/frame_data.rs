use ash::vk;

use super::{deletion_queue::DeletionQueue, device::Device, error::VulkanError};
use crate::vulkan_error;

#[derive(Default)]
pub struct FrameData {
  pub deletion_queue: DeletionQueue,
  pub command_pool: vk::CommandPool,
  pub master_command_buffer: vk::CommandBuffer,
  pub render_fence: vk::Fence,
  pub present_semaphore: vk::Semaphore,
  pub render_semaphore: vk::Semaphore,
}

impl FrameData {
  pub const FRAME_OVERLAP: usize = 2;

  pub fn new(device: &Device) -> Result<FrameData, VulkanError> {
    let deletion_queue = DeletionQueue::new();

    let create_info = vk::CommandPoolCreateInfo::builder()
      .queue_family_index(device.graphics().family())
      .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    let command_pool = unsafe { device.logical().create_command_pool(&create_info, None) }?;

    let buffer_info = vk::CommandBufferAllocateInfo::builder()
      .command_pool(command_pool)
      .command_buffer_count(1)
      .level(vk::CommandBufferLevel::PRIMARY);

    let master_command_buffer = unsafe { device.logical().allocate_command_buffers(&buffer_info) }?
      .first()
      .cloned()
      .ok_or_else(|| vulkan_error!("invalid command buffers size"))?;

    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let render_fence = unsafe { device.logical().create_fence(&fence_info, None) }?;

    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let swapchain_semaphore = unsafe { device.logical().create_semaphore(&semaphore_info, None) }?;
    let render_semaphore = unsafe { device.logical().create_semaphore(&semaphore_info, None) }?;

    Ok(FrameData {
      deletion_queue,
      command_pool,
      master_command_buffer,
      render_fence,
      present_semaphore: swapchain_semaphore,
      render_semaphore,
    })
  }

  pub fn delete(&mut self, device: &mut Device) {
    unsafe {
      device.logical().destroy_command_pool(self.command_pool, None);
      device.logical().destroy_fence(self.render_fence, None);
      device.logical().destroy_semaphore(self.present_semaphore, None);
      device.logical().destroy_semaphore(self.render_semaphore, None);
    }
  }
}
