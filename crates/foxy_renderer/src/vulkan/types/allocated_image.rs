use ash::vk;

pub struct AllocatedImage {
  pub image: vk::Image,
  pub view: vk::ImageView,
  pub allocation: vk_mem::Allocation,
  pub extent: vk::Extent3D,
  pub format: vk::Format,
}

