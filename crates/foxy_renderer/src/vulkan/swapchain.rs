use std::sync::Arc;

use itertools::Itertools;
use tracing::*;
use vulkano::{
  device::Device,
  format::Format,
  image::{view::ImageView, Image, ImageUsage},
  pipeline::graphics::viewport::Viewport,
  swapchain::{ColorSpace, Surface, Swapchain, SwapchainCreateInfo},
  sync::semaphore::Semaphore,
};
use winit::dpi::PhysicalSize;

use crate::vulkan::{error::VulkanError, swapchain::image_format::PresentMode};

pub struct FoxySwapchain {
  swapchain: Arc<Swapchain>,
  images: Vec<Arc<Image>>,
  image_views: Vec<Arc<ImageView>>,
  viewport: Viewport,
}

pub mod image_format;

impl FoxySwapchain {
  pub fn new(
    surface: Arc<Surface>,
    device: Arc<Device>,
    dims: PhysicalSize<u32>,
    preferred_present_mode: PresentMode,
  ) -> Result<Self, VulkanError> {
    let (swapchain, images) = {
      let surface_caps = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())?;

      let surface_formats = device.physical_device().surface_formats(&surface, Default::default())?;

      // debug!("Available formats: {surface_formats:#?}");

      let (format, color_space) = surface_formats
        .iter()
        .filter(|(f, c)| f == &Format::B8G8R8A8_UNORM && c == &ColorSpace::SrgbNonLinear)
        .collect_vec()
        .first()
        .unwrap();

      debug!("Selected format: [{format:#?}, {color_space:#?}]");

      let present_modes = device
        .physical_device()
        .surface_present_modes(&surface, Default::default())?
        .collect_vec();

      let create_info = SwapchainCreateInfo {
        min_image_count: surface_caps.min_image_count.max(2),
        image_color_space: *color_space,
        image_format: *format,
        image_extent: dims.into(),
        present_mode: preferred_present_mode.select_from(present_modes),
        image_usage: ImageUsage::COLOR_ATTACHMENT,
        composite_alpha: surface_caps.supported_composite_alpha.into_iter().next().unwrap(),
        ..Default::default()
      };

      Swapchain::new(device, surface, create_info)?
    };

    let mut viewport = Viewport {
      offset: [0.0, 0.0],
      extent: [0.0, 0.0],
      depth_range: 0.0..=1.0,
    };

    let image_views = Self::window_size_dependent_setup(&images, &mut viewport);

    Ok(Self {
      swapchain,
      images,
      image_views,
      viewport,
    })
  }

  fn window_size_dependent_setup(images: &[Arc<Image>], viewport: &mut Viewport) -> Vec<Arc<ImageView>> {
    let extent = images.first().unwrap().extent();
    viewport.extent = [*extent.first().unwrap() as f32, *extent.last().unwrap() as f32];

    images
      .iter()
      .map(|image| ImageView::new_default(image.clone()).unwrap())
      .collect_vec()
  }

  pub fn extent(&self) -> (u32, u32) {
    let extent = self.swapchain.image_extent();
    (*extent.first().unwrap(), *extent.last().unwrap())
  }

  pub fn aspect_ratio(&self) -> f32 {
    let extent = self.swapchain.image_extent();
    *extent.first().unwrap() as f32 / *extent.last().unwrap() as f32
  }

  pub fn vk(&self) -> &Arc<Swapchain> {
    &self.swapchain
  }

  pub fn image_count(&self) -> usize {
    self.images.len()
  }

  pub fn image(&self, index: usize) -> Option<Arc<Image>> {
    self.images.get(index).cloned()
  }

  pub fn acquire_next_image(&mut self, semaphore: Semaphore) -> Result<(usize, bool), VulkanError> {
    todo!()
  }
}
