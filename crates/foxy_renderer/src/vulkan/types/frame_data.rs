use std::sync::Arc;

use itertools::Itertools;
use vulkano::{
  command_buffer::{
    allocator::{CommandBufferAllocator, CommandBufferBuilderAlloc, StandardCommandBufferAllocator},
    pool::{CommandBufferAllocateInfo, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo},
    AutoCommandBufferBuilder,
    CommandBufferLevel,
    CommandBufferUsage,
    PrimaryAutoCommandBuffer,
  },
  device::{Device, DeviceOwned, Queue},
  sync::{
    self,
    fence::{Fence, FenceCreateFlags, FenceCreateInfo},
    semaphore::{Semaphore, SemaphoreCreateInfo},
    GpuFuture,
  },
};

use crate::{
  vulkan::{device::FoxyDevice, error::VulkanError},
  vulkan_error,
};

pub struct FrameData {
  pub cmd_buffer_allocator: Arc<StandardCommandBufferAllocator>,
  pub imm_cmd_buffer_allocator: Arc<StandardCommandBufferAllocator>,

  // pub render_fence: Fence,
  // pub imm_fence: Fence,
  // pub present_semaphore: Semaphore,
  // pub render_semaphore: Semaphore,

  pub previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl FrameData {
  pub const FRAME_OVERLAP: usize = 2;

  pub fn new(device: &FoxyDevice) -> Result<FrameData, VulkanError> {
    // init command pool

    let cmd_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(device.vk().clone(), Default::default()));
    let imm_cmd_buffer_allocator =
      Arc::new(StandardCommandBufferAllocator::new(device.vk().clone(), Default::default()));

    let create_info = CommandPoolCreateInfo {
      flags: CommandPoolCreateFlags::TRANSIENT | CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
      queue_family_index: device.graphics_queue().queue_family_index(),
      ..Default::default()
    };

    // init sync objects

    let fence_info = FenceCreateInfo {
      flags: FenceCreateFlags::SIGNALED,
      ..Default::default()
    };
    // let render_fence = Fence::new(device.vk().clone(), fence_info)?;
    // let imm_fence = Fence::new(device.vk().clone(), fence_info)?;

    // let semaphore_info = SemaphoreCreateInfo::default();
    // let swapchain_semaphore = Semaphore::new(device.vk().clone(), semaphore_info)?;
    // let render_semaphore = Semaphore::new(device.vk().clone(), semaphore_info)?;

    let previous_frame_end = Some(sync::now(device.vk().clone()).boxed());

    Ok(FrameData {
      cmd_buffer_allocator,
      imm_cmd_buffer_allocator,
      // render_fence,
      // present_semaphore: swapchain_semaphore,
      // render_semaphore,
      // imm_fence,
      previous_frame_end,
    })
  }

  pub fn primary_command(
    &self,
    queue: &Arc<Queue>,
  ) -> Result<
    AutoCommandBufferBuilder<
      PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>,
      Arc<StandardCommandBufferAllocator>,
    >,
    VulkanError,
  > {
    let builder = AutoCommandBufferBuilder::primary(
      &self.cmd_buffer_allocator,
      queue.queue_family_index(),
      CommandBufferUsage::OneTimeSubmit,
    )?;

    Ok(builder)
  }
}
