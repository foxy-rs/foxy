use std::sync::Arc;

use ash::{extensions::khr, vk};

use crate::{error::VulkanError, image::Image, surface::Surface};

pub struct Swapchain {
  device: Arc<ash::Device>,

  current_frame_index: u32,
  images_in_flight: Vec<vk::Fence>,
  fences_in_flight: Vec<vk::Fence>,
  image_avaiable_semaphores: Vec<vk::Semaphore>,
  render_finished_semaphores: Vec<vk::Semaphore>,

  window_extent: vk::Extent2D,

  swapchain_image_views: Vec<vk::ImageView>,
  swapchain_images: Vec<vk::Image>,

  depth_images: Vec<Image>,

  render_pass: vk::RenderPass,
  swapchain_framebuffers: Vec<vk::Framebuffer>,

  swapchain_extent: vk::Extent2D,
  swapchain_image_format: vk::Format,

  swapchain: vk::SwapchainKHR,
  swapchain_loader: khr::Swapchain,
}

impl Swapchain {
  const MAX_FRAMES_IN_FLIGHT: u32 = 2;

  pub fn new(instance: &ash::Instance, device: Arc<ash::Device>, surface: &Surface) -> Result<Self, VulkanError> {
    let swapchain_loader = unsafe { khr::Swapchain::new(instance, &device) };
    let swapchain = Self::create_swap_chain(&swapchain_loader, surface)?;

    Ok(Self {
      device,
      swapchain,
      current_frame_index: 0,
      images_in_flight: todo!(),
      fences_in_flight: todo!(),
      image_avaiable_semaphores: todo!(),
      render_finished_semaphores: todo!(),
      window_extent: todo!(),
      swapchain_image_views: todo!(),
      swapchain_images: todo!(),
      depth_images: todo!(),
      render_pass: todo!(),
      swapchain_framebuffers: todo!(),
      swapchain_extent: todo!(),
      swapchain_image_format: todo!(),
      swapchain_loader,
    })
  }

  fn create_swap_chain(swapchain_loader: &khr::Swapchain, surface: &Surface) -> Result<vk::SwapchainKHR, VulkanError> {
    let create_info = vk::SwapchainCreateInfoKHR::default().surface(*surface.surface());

    let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;

    Ok(swapchain)
  }

  fn create_image_views() {}

  fn create_depth_resources() {}

  fn create_render_pass() {}

  fn create_framebuffers() {}

  fn create_sync_objects() {}
}

impl Drop for Swapchain {
  fn drop(&mut self) {
    unsafe {}
  }
}
