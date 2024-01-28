use std::{mem::ManuallyDrop, ops::DerefMut, sync::Arc};

use ash::vk;

use crate::{device::Device, error::VulkanError, swapchain::Swapchain};

pub struct SyncObjects {
  device: Arc<Device>,
  pub images_in_flight: ManuallyDrop<Vec<vk::Fence>>,
  pub fences_in_flight: ManuallyDrop<Vec<vk::Fence>>,
  pub image_avaiable_semaphores: ManuallyDrop<Vec<vk::Semaphore>>,
  pub render_finished_semaphores: ManuallyDrop<Vec<vk::Semaphore>>,
}

impl Drop for SyncObjects {
  fn drop(&mut self) {
    unsafe {
      // sync objects
      for i in 0..Swapchain::MAX_FRAMES_IN_FLIGHT {
        self
          .device
          .logical()
          .destroy_semaphore(self.render_finished_semaphores[i], None);
        self
          .device
          .logical()
          .destroy_semaphore(self.image_avaiable_semaphores[i], None);
        self.device.logical().destroy_fence(self.fences_in_flight[i], None);
        self.device.logical().destroy_fence(self.images_in_flight[i], None);
      }
      ManuallyDrop::drop(&mut self.render_finished_semaphores);
      ManuallyDrop::drop(&mut self.image_avaiable_semaphores);
      ManuallyDrop::drop(&mut self.fences_in_flight);
      ManuallyDrop::drop(&mut self.images_in_flight);
    }
  }
}

impl SyncObjects {
  pub fn new(device: Arc<Device>) -> Result<Self, VulkanError> {
    let sema_info = vk::SemaphoreCreateInfo::default();
    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

    let mut image_avaiable_semaphores = ManuallyDrop::new(vec![Default::default(); Swapchain::MAX_FRAMES_IN_FLIGHT]);
    for fence in image_avaiable_semaphores.deref_mut() {
      *fence = unsafe { device.logical().create_semaphore(&sema_info, None) }?;
    }

    let mut render_finished_semaphores = ManuallyDrop::new(vec![Default::default(); Swapchain::MAX_FRAMES_IN_FLIGHT]);
    for fence in render_finished_semaphores.deref_mut() {
      *fence = unsafe { device.logical().create_semaphore(&sema_info, None) }?;
    }

    let mut fences_in_flight = ManuallyDrop::new(vec![Default::default(); Swapchain::MAX_FRAMES_IN_FLIGHT]);
    for fence in fences_in_flight.deref_mut() {
      *fence = unsafe { device.logical().create_fence(&fence_info, None) }?;
    }

    let images_in_flight = ManuallyDrop::new(vec![Default::default(); Swapchain::MAX_FRAMES_IN_FLIGHT]);

    Ok(Self { device, images_in_flight, fences_in_flight, image_avaiable_semaphores, render_finished_semaphores })
  }
}
