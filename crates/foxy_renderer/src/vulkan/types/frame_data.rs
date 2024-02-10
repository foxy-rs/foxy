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

pub type PrimaryCommandBufferBuilder = AutoCommandBufferBuilder<
  PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>,
  Arc<StandardCommandBufferAllocator>,
>;

pub struct FrameData {
  pub cmd_buffer_allocator: Arc<StandardCommandBufferAllocator>,
  pub imm_cmd_buffer_allocator: Arc<StandardCommandBufferAllocator>,

  // pub render_fence: Fence,
  // pub imm_fence: Fence,
  // pub present_semaphore: Semaphore,
  // pub render_semaphore: Semaphore,
}

impl FrameData {
  pub const FRAME_OVERLAP: usize = 2;

  pub fn new(device: &FoxyDevice) -> Result<FrameData, VulkanError> {
    // init command pool

    let cmd_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(device.vk().clone(), Default::default()));
    let imm_cmd_buffer_allocator =
      Arc::new(StandardCommandBufferAllocator::new(device.vk().clone(), Default::default()));

    Ok(FrameData {
      cmd_buffer_allocator,
      imm_cmd_buffer_allocator,
    })
  }

  pub fn primary_command(&self, queue: &Arc<Queue>) -> Result<PrimaryCommandBufferBuilder, VulkanError> {
    let builder = AutoCommandBufferBuilder::primary(
      &self.cmd_buffer_allocator,
      queue.queue_family_index(),
      CommandBufferUsage::OneTimeSubmit,
    )?;

    Ok(builder)
  }
}
