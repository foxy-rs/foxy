use ash::vk;
use vk_mem::Allocator;

use crate::vulkan::device::Device;

pub struct AllocatedImage {
  pub image: vk::Image,
  pub view: vk::ImageView,
  pub allocation: vk_mem::Allocation,
  pub extent: vk::Extent3D,
  pub format: vk::Format,
}

impl AllocatedImage {
  pub fn delete(&mut self, device: &Device, allocator: &Allocator) {
    unsafe { device.logical().destroy_image_view(self.view, None) };
    unsafe { allocator.destroy_image(self.image, &mut self.allocation) };
  }
}
