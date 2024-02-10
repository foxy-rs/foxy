use std::sync::Arc;

use itertools::Itertools;
use tracing::*;
use vulkano::{
  device::Device,
  format::Format,
  image::{
    view::{ImageView, ImageViewCreateInfo},
    Image,
    ImageCreateInfo,
    ImageLayout,
    ImageSubresourceRange,
    ImageTiling,
    ImageType,
    ImageUsage,
    SampleCount,
  },
  memory::allocator::{AllocationCreateInfo, MemoryAllocatePreference, MemoryTypeFilter, StandardMemoryAllocator},
  pipeline::graphics::viewport::Viewport,
  swapchain::{acquire_next_image, ColorSpace, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo},
  sync::semaphore::Semaphore,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::vulkan::{error::VulkanError, swapchain::image_format::PresentMode};

pub struct FoxySwapchain {
  window: Arc<Window>,
  viewport: Viewport,

  swapchain: Arc<Swapchain>,

  images: Vec<Arc<Image>>,
  image_views: Vec<Arc<ImageView>>,

  draw_image: Arc<Image>,
  draw_image_view: Arc<ImageView>,
}

pub mod image_format;

impl FoxySwapchain {
  pub fn new(
    window: Arc<Window>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    allocator: Arc<StandardMemoryAllocator>,
    preferred_present_mode: PresentMode,
  ) -> Result<Self, VulkanError> {
    let inner_size = window.inner_size();
    let extent_2d = [inner_size.width, inner_size.height];
    let extent_3d = [inner_size.width, inner_size.height, 1];
    let mut viewport = Viewport {
      offset: [0.0, 0.0],
      extent: [0.0, 0.0],
      depth_range: 0.0..=1.0,
    };

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
        image_extent: extent_2d,
        present_mode: preferred_present_mode.select_from(present_modes),
        image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
        composite_alpha: surface_caps.supported_composite_alpha.into_iter().next().unwrap(),
        ..Default::default()
      };

      Swapchain::new(device, surface, create_info)?
    };

    let image_views = Self::generate_image_views(&images, &mut viewport);

    let draw_image = Image::new(
      allocator.clone(),
      ImageCreateInfo {
        image_type: ImageType::Dim2d,
        format: Format::R16G16B16A16_SFLOAT,
        extent: extent_3d,
        mip_levels: 1,
        array_layers: 1,
        samples: SampleCount::Sample1,
        tiling: ImageTiling::Optimal,
        usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC | ImageUsage::TRANSFER_DST | ImageUsage::STORAGE,
        initial_layout: ImageLayout::Undefined,
        ..Default::default()
      },
      AllocationCreateInfo {
        allocate_preference: MemoryAllocatePreference::AlwaysAllocate,
        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
        ..Default::default()
      },
    )?;

    let draw_image_view = ImageView::new_default(draw_image.clone())?;

    Ok(Self {
      window,
      viewport,
      swapchain,
      images,
      image_views,
      draw_image,
      draw_image_view,
    })
  }

  fn generate_image_views(images: &[Arc<Image>], viewport: &mut Viewport) -> Vec<Arc<ImageView>> {
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

  pub fn viewport(&self) -> &Viewport {
    &self.viewport
  }

  pub fn image_count(&self) -> usize {
    self.images.len()
  }

  pub fn image(&self, index: usize) -> Arc<Image> {
    self.images.get(index).cloned().expect("invalid swapchain image index")
  }

  pub fn image_view(&self, index: usize) -> Arc<ImageView> {
    self
      .image_views
      .get(index)
      .cloned()
      .expect("invalid swapchain image index")
  }

  pub fn draw_image(&self) -> Arc<Image> {
    self.draw_image.clone()
  }

  pub fn draw_image_view(&self) -> Arc<ImageView> {
    self.draw_image_view.clone()
  }

  pub fn acquire_next_image(&mut self) -> Result<(u32, bool, SwapchainAcquireFuture), VulkanError> {
    Ok(acquire_next_image(self.swapchain.clone(), None)?)
  }

  pub fn rebuild(&mut self) -> Result<(), VulkanError> {
    let image_extent: [u32; 2] = self.window.inner_size().into();
    (self.swapchain, self.images) = self.swapchain.recreate(SwapchainCreateInfo {
      image_extent,
      ..self.swapchain.create_info()
    })?;

    self.image_views = Self::generate_image_views(&self.images, &mut self.viewport);

    Ok(())
  }
}
