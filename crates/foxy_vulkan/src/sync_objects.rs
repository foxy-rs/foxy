use std::ops::DerefMut;

use ash::vk;
use foxy_types::handle::Handle;

use crate::{device::Device, error::VulkanError, swapchain::Swapchain};

pub struct SyncObjects {
  device: Handle<Device>,
  pub images_in_flight: Vec<vk::Fence>,
  pub fences_in_flight: Vec<vk::Fence>,
  pub image_avaiable_semaphores: Vec<vk::Semaphore>,
  pub render_finished_semaphores: Vec<vk::Semaphore>,
}

impl SyncObjects {
  pub fn delete(&mut self) {
    unsafe {
      // sync objects
      for i in 0..Swapchain::MAX_FRAMES_IN_FLIGHT {
        self
          .device
          .get()
          .logical()
          .destroy_semaphore(self.render_finished_semaphores[i], None);
        self
          .device
          .get()
          .logical()
          .destroy_semaphore(self.image_avaiable_semaphores[i], None);
        self
          .device
          .get()
          .logical()
          .destroy_fence(self.fences_in_flight[i], None);
        self
          .device
          .get()
          .logical()
          .destroy_fence(self.images_in_flight[i], None);
      }
    }
  }
}

impl SyncObjects {
  pub fn new(device: Handle<Device>) -> Result<Self, VulkanError> {
    let sema_info = vk::SemaphoreCreateInfo::default();
    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

    let mut image_avaiable_semaphores = vec![Default::default(); Swapchain::MAX_FRAMES_IN_FLIGHT];
    for fence in image_avaiable_semaphores.deref_mut() {
      *fence = unsafe { device.get().logical().create_semaphore(&sema_info, None) }?;
    }

    let mut render_finished_semaphores = vec![Default::default(); Swapchain::MAX_FRAMES_IN_FLIGHT];
    for fence in render_finished_semaphores.deref_mut() {
      *fence = unsafe { device.get().logical().create_semaphore(&sema_info, None) }?;
    }

    let mut fences_in_flight = vec![Default::default(); Swapchain::MAX_FRAMES_IN_FLIGHT];
    for fence in fences_in_flight.deref_mut() {
      *fence = unsafe { device.get().logical().create_fence(&fence_info, None) }?;
    }

    let images_in_flight = vec![Default::default(); Swapchain::MAX_FRAMES_IN_FLIGHT];

    Ok(Self {
      device,
      images_in_flight,
      fences_in_flight,
      image_avaiable_semaphores,
      render_finished_semaphores,
    })
  }
}
