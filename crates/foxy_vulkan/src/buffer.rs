use std::{mem::ManuallyDrop, sync::Arc};

use anyhow::Context;
use ash::{self, vk};

use crate::{device::Device, error::VulkanError, image::Image};

pub struct Buffer {
  device: Arc<Device>,
  buffer: ManuallyDrop<vk::Buffer>,
  memory: ManuallyDrop<vk::DeviceMemory>,
  size: vk::DeviceSize,
}

impl Buffer {
  pub fn new(
    device: Arc<Device>,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
  ) -> Result<Self, VulkanError> {
    let buffer_create_info = vk::BufferCreateInfo {
      size,
      usage,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let mut buffer = ManuallyDrop::new(
      unsafe { device.logical().create_buffer(&buffer_create_info, None) }.context("Failed to create buffer")?,
    );

    let memory_reqs = unsafe { device.logical().get_buffer_memory_requirements(*buffer) };

    let memory_create_info = vk::MemoryAllocateInfo {
      allocation_size: memory_reqs.size,
      memory_type_index: device
        .find_memory_type(memory_reqs.memory_type_bits, properties)
        .heap_index,
      ..Default::default()
    }
    .allocation_size(memory_reqs.size);

    let memory = ManuallyDrop::new(
      match unsafe { device.logical().allocate_memory(&memory_create_info, None) }
        .context("Failed to allocate buffer memory")
      {
        Ok(value) => value,
        Err(err) => unsafe {
          device.logical().destroy_buffer(*buffer, None);
          ManuallyDrop::drop(&mut buffer);
          Err(err)?
        },
      },
    );

    Ok(Self {
      device,
      buffer,
      memory,
      size,
    })
  }

  pub fn buffer(&self) -> vk::Buffer {
    *self.buffer
  }

  pub fn memory(&self) -> vk::DeviceMemory {
    *self.memory
  }

  pub fn size(&self) -> vk::DeviceSize {
    self.size
  }

  // unsafe fn delete(&mut self) {
  //   unsafe {
  //     self.device.destroy_buffer(self.buffer, None);
  //     self.device.free_memory(self.memory, None);
  //   }
  // }

  pub fn copy_to_buffer(&self, vulkan: &Device, dst: &Buffer) {
    vulkan.issue_single_time_commands(|command_buffer| {
      let copy_region = vk::BufferCopy::default().size(self.size);

      unsafe {
        self
          .device
          .logical()
          .cmd_copy_buffer(command_buffer, *self.buffer, *dst.buffer, &[copy_region]);
      }
    });
  }

  pub fn copy_to_image(&self, vulkan: &Device, image: &Image) {
    vulkan.issue_single_time_commands(|command_buffer| {
      let copy_region = vk::BufferImageCopy::default()
        .image_subresource(
          vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(image.layer_count()),
        )
        .image_extent(
          vk::Extent3D::default()
            .width(image.extent().width)
            .height(image.extent().height)
            .depth(1),
        );

      unsafe {
        self.device.logical().cmd_copy_buffer_to_image(
          command_buffer,
          *self.buffer,
          image.image(),
          vk::ImageLayout::TRANSFER_DST_OPTIMAL,
          &[copy_region],
        );
      }
    });
  }
}

impl Drop for Buffer {
  fn drop(&mut self) {
    unsafe {
      self.device.logical().destroy_buffer(*self.buffer, None);
      ManuallyDrop::drop(&mut self.buffer);

      self.device.logical().free_memory(*self.memory, None);
      ManuallyDrop::drop(&mut self.memory);
    }
  }
}
