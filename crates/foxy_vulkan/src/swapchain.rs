use std::{mem::ManuallyDrop, sync::Arc};

use ash::{extensions::khr, vk};
use foxy_util::log::LogErr;

use crate::{error::VulkanError, image::Image, Vulkan};

pub struct Swapchain {
  device: Arc<ash::Device>,

  current_frame_index: u32,
  images_in_flight: Vec<ManuallyDrop<vk::Fence>>,
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

impl Drop for Swapchain {
  fn drop(&mut self) {
    unsafe {
      self.swapchain_loader.destroy_swapchain(self.swapchain, None);
    }
  }
}

impl Iterator for Swapchain {
  type Item = u32;

  fn next(&mut self) -> Option<Self::Item> {
    self.acquire_next_image().log_error().ok()
  }
}

impl Swapchain {
  const MAX_FRAMES_IN_FLIGHT: u32 = 2;

  pub fn new(vulkan: &Vulkan) -> Result<Self, VulkanError> {
    let swapchain_loader = khr::Swapchain::new(vulkan.instance(), &vulkan.logical());
    let swapchain = Self::create_swap_chain(&swapchain_loader, vulkan)?;

    Ok(Self {
      device: vulkan.logical(),
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

  pub fn submit_command_buffers(&mut self, buffers: &[vk::CommandBuffer], image_index: u32) -> Result<(), VulkanError> {
    todo!()
  }

  pub fn acquire_next_image(&mut self) -> Result<u32, VulkanError> {
    todo!()
  }

  pub fn find_depth_format(&self) -> vk::Format {
    todo!()
  }

  pub fn extent_aspect_ratio(&self) -> f32 {
    self.width() as f32 / self.height() as f32
  }

  pub fn width(&self) -> u32 {
    self.swapchain_extent.width
  }

  pub fn height(&self) -> u32 {
    self.swapchain_extent.height
  }

  pub fn swapchain_image_format(&self) -> vk::Format {
    self.swapchain_image_format
  }

  pub fn image_count(&self) -> usize {
    self.swapchain_images.len()
  }

  pub fn image_view(&self, index: usize) -> Option<&vk::ImageView> {
    self.swapchain_image_views.get(index)
  }

  pub fn render_pass(&self) -> vk::RenderPass {
    self.render_pass
  }

  pub fn frame_buffer(&self, index: usize) -> Option<&vk::Framebuffer> {
    self.swapchain_framebuffers.get(index)
  }

  fn create_swap_chain(swapchain_loader: &khr::Swapchain, vulkan: &Vulkan) -> Result<vk::SwapchainKHR, VulkanError> {
    let create_info = vk::SwapchainCreateInfoKHR::default().surface(*vulkan.surface().surface());

    let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;

    Ok(swapchain)
  }

  fn create_image_views() {}

  fn create_depth_resources() {}

  fn create_render_pass() {}

  fn create_framebuffers() {}

  fn create_sync_objects() {}

  fn choose_swap_surface_format(available_formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
    todo!()
  }

  fn choose_swap_present_mode(available_formats: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    todo!()
  }

  fn choose_swap_extent(available_formats: Vec<vk::SurfaceCapabilitiesKHR>) -> vk::SurfaceCapabilitiesKHR {
    todo!()
  }
}
