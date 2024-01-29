use std::{mem::ManuallyDrop, sync::Arc};

use anyhow::Context;
use ash::{self, vk};
use foxy_types::handle::Handle;

use crate::{device::Device, error::VulkanError};

pub struct Image {
  device: Arc<ash::Device>,
  image: vk::Image,
  memory: vk::DeviceMemory,
  extent: vk::Extent3D,
  layer_count: u32,
}

impl Image {
  pub fn delete(&mut self) {
    unsafe {
      self.device.destroy_image(self.image, None);
      self.device.free_memory(self.memory, None);
    }
  }
}

impl Image {
  pub fn new(
    device: Handle<Device>,
    image_info: vk::ImageCreateInfo,
    properties: vk::MemoryPropertyFlags,
  ) -> Result<Self, VulkanError> {
    let mut image = unsafe { device.get().logical().create_image(&image_info, None) }.context("Failed to create image")?;

    let memory_reqs = unsafe { device.get().logical().get_image_memory_requirements(image) };

    let allocation_info = vk::MemoryAllocateInfo::default()
      .memory_type_index(
        device.get()
          .find_memory_type(memory_reqs.memory_type_bits, properties)
          .heap_index,
      )
      .allocation_size(memory_reqs.size);

    let mut memory = match unsafe { device.get().logical().allocate_memory(&allocation_info, None) }
      .context("Failed to allocate memory for image")
    {
      Ok(value) => value,
      Err(err) => unsafe {
        device.get().logical().destroy_image(image, None);
        Err(err)?
      },
    };

    if let Err(err) =
      unsafe { device.get().logical().bind_image_memory(image, memory, 0) }.context("Failed to bind image memory")
    {
      unsafe {
        device.get().logical().destroy_image(image, None);
        device.get().logical().free_memory(memory, None);
      }
      Err(err)?
    };

    Ok(Self {
      device: device.get().logical(),
      image,
      memory,
      extent: image_info.extent,
      layer_count: image_info.array_layers,
    })
  }

  // unsafe fn delete(&mut self) {
  //   unsafe {
  //     self.device.destroy_image(self.image, None);
  //     self.device.free_memory(self.memory, None);
  //   }
  // }

  pub fn image(&self) -> vk::Image {
    self.image
  }

  pub fn memory(&self) -> vk::DeviceMemory {
    self.memory
  }

  pub fn extent(&self) -> vk::Extent3D {
    self.extent
  }

  pub fn layer_count(&self) -> u32 {
    self.layer_count
  }
}
