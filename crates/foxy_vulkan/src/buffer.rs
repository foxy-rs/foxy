use anyhow::{Context, Result};
use ash::{self, vk};
use std::sync::Arc;

use crate::{image::Image, Vulkan};

pub struct Buffer {
  device: Arc<ash::Device>,
  pub buffer: vk::Buffer,
  pub memory: vk::DeviceMemory,
  pub size: vk::DeviceSize,
}

impl Buffer {
  pub fn new(
    vulkan: &Vulkan,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
  ) -> Result<Self> {
    let buffer_create_info = vk::BufferCreateInfo {
      size,
      usage,
      sharing_mode: vk::SharingMode::EXCLUSIVE,
      ..Default::default()
    };

    let buffer =
      unsafe { vulkan.logical().create_buffer(&buffer_create_info, None) }.context("Failed to create buffer")?;

    let memory_reqs = unsafe { vulkan.logical().get_buffer_memory_requirements(buffer) };

    let memory_create_info = vk::MemoryAllocateInfo {
      allocation_size: memory_reqs.size,
      memory_type_index: vulkan
        .find_memory_type(memory_reqs.memory_type_bits, properties)
        .heap_index,
      ..Default::default()
    };

    let memory = match unsafe { vulkan.logical().allocate_memory(&memory_create_info, None) }
      .context("Failed to allocate buffer memory")
    {
      Ok(value) => value,
      Err(err) => unsafe {
        vulkan.logical().destroy_buffer(buffer, None);
        Err(err)?
      },
    };

    Ok(Self {
      device: vulkan.logical(),
      buffer,
      memory,
      size,
    })
  }

  // unsafe fn delete(&mut self) {
  //   unsafe {
  //     self.device.destroy_buffer(self.buffer, None);
  //     self.device.free_memory(self.memory, None);
  //   }
  // }

  pub fn copy_to_buffer(&self, vulkan: &Vulkan, dst: &Buffer) {
    vulkan.issue_single_time_commands(|command_buffer| {
      let copy_region = vk::BufferCopy {
        size: self.size,
        ..Default::default()
      };

      unsafe {
        self
          .device
          .cmd_copy_buffer(command_buffer, self.buffer, dst.buffer, &[copy_region]);
      }
    });
  }

  pub fn copy_to_image(&self, vulkan: &Vulkan, image: &Image) {
    vulkan.issue_single_time_commands(|command_buffer| {
      let copy_region = vk::BufferImageCopy {
        image_subresource: vk::ImageSubresourceLayers {
          aspect_mask: vk::ImageAspectFlags::COLOR,
          layer_count: image.layer_count,
          ..Default::default()
        },
        image_extent: vk::Extent3D {
          width: image.extent.width,
          height: image.extent.height,
          depth: 1,
        },
        ..Default::default()
      };

      unsafe {
        self.device.cmd_copy_buffer_to_image(
          command_buffer,
          self.buffer,
          image.image,
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
      self.device.destroy_buffer(self.buffer, None);
      self.device.free_memory(self.memory, None);
    }
  }
}
