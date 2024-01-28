use std::{mem::ManuallyDrop, sync::Arc};

use anyhow::{Context, Result};
use ash::{self, vk};

use crate::Vulkan;

pub struct Image {
  device: Arc<ash::Device>,
  pub image: ManuallyDrop<vk::Image>,
  pub memory: ManuallyDrop<vk::DeviceMemory>,
  pub extent: vk::Extent3D,
  pub layer_count: u32,
}

impl Image {
  pub fn new(vulkan: &Vulkan, image_info: vk::ImageCreateInfo, properties: vk::MemoryPropertyFlags) -> Result<Self> {
    let mut image =
      ManuallyDrop::new(unsafe { vulkan.logical().create_image(&image_info, None) }.context("Failed to create image")?);

    let memory_reqs = unsafe { vulkan.logical().get_image_memory_requirements(*image) };

    let allocation_info = vk::MemoryAllocateInfo::default().memory_type_index(
      vulkan
        .find_memory_type(memory_reqs.memory_type_bits, properties)
        .heap_index,
    );

    let mut memory = ManuallyDrop::new(
      match unsafe { vulkan.logical().allocate_memory(&allocation_info, None) }
        .context("Failed to allocate memory for image")
      {
        Ok(value) => value,
        Err(err) => unsafe {
          vulkan.logical().destroy_image(*image, None);
          ManuallyDrop::drop(&mut image);
          Err(err)?
        },
      },
    );

    if let Err(err) =
      unsafe { vulkan.logical().bind_image_memory(*image, *memory, 0) }.context("Failed to bind image memory")
    {
      unsafe {
        vulkan.logical().destroy_image(*image, None);
        ManuallyDrop::drop(&mut image);
        vulkan.logical().free_memory(*memory, None);
        ManuallyDrop::drop(&mut memory);
      }
      Err(err)?
    };

    Ok(Self {
      device: vulkan.logical(),
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
}

impl Drop for Image {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_image(*self.image, None);
      ManuallyDrop::drop(&mut self.image);
      self.device.free_memory(*self.memory, None);
      ManuallyDrop::drop(&mut self.memory);
    }
  }
}
