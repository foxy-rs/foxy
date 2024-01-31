use ash::{extensions::khr, vk};
use foxy_utils::types::{handle::Handle, primitives::Dimensions};
use tracing::debug;

use self::image_format::{ColorSpace, ImageFormat, PresentMode};
use super::{device::Device, instance::Instance, surface::Surface};
use crate::vulkan::error::VulkanError;

pub struct Swapchain {
  device: Handle<Device>,

  extent: vk::Extent2D,
  image_format: vk::Format,

  image_views: Vec<vk::ImageView>,
  images: Vec<vk::Image>,

  swapchain: vk::SwapchainKHR,
  swapchain_loader: khr::Swapchain,
}

pub mod buffer;
pub mod image;
pub mod image_format;
pub mod pipeline;

impl Swapchain {
  pub fn delete(&mut self) {
    unsafe {
      // image views
      for &image_view in self.image_views.iter() {
        self.device.get().logical().destroy_image_view(image_view, None);
      }
      self.image_views.clear();

      // swapchain
      self.swapchain_loader.destroy_swapchain(self.swapchain, None);
    }
  }
}

impl Swapchain {
  pub fn new(
    instance: &Handle<Instance>,
    surface: &Surface,
    device: Handle<Device>,
    extent: Dimensions,
    preferred_image_format: ImageFormat,
  ) -> Result<Self, VulkanError> {
    debug!("Window extent: {extent:?}");
    let extent = vk::Extent2D::default()
      .width(extent.width as u32)
      .height(extent.height as u32);
    debug!("Window extent (true): {extent:?}");

    let swapchain_loader = khr::Swapchain::new(instance.get().raw(), device.get().logical());
    let (swapchain, images, image_format) =
      Self::create_swap_chain(surface, device.clone(), &swapchain_loader, preferred_image_format, extent)?;

    let image_views = Self::create_image_views(device.clone(), &images, image_format)?;

    Ok(Self {
      device,
      swapchain,
      image_views,
      images,
      extent,
      image_format,
      swapchain_loader,
    })
  }

  pub fn aspect_ratio(&self) -> f32 {
    self.size().width as f32 / self.size().height as f32
  }

  pub fn size(&self) -> Dimensions {
    Dimensions {
      width: self.extent.width as i32,
      height: self.extent.height as i32,
    }
  }

  pub fn image_format(&self) -> vk::Format {
    self.image_format
  }

  pub fn image_count(&self) -> usize {
    self.images.len()
  }

  pub fn image_view(&self, index: usize) -> Option<vk::ImageView> {
    self.image_views.get(index).cloned()
  }

  fn create_swap_chain(
    surface: &Surface,
    device: Handle<Device>,
    swapchain_loader: &khr::Swapchain,
    preferred_image_format: ImageFormat,
    window_extent: vk::Extent2D,
  ) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format), VulkanError> {
    let swapchain_support = surface.swapchain_support(*device.get().physical())?;

    let surface_format =
      Self::choose_swap_surface_format(swapchain_support.formats, preferred_image_format.color_space);
    let present_mode =
      Self::choose_swap_present_mode(swapchain_support.present_modes, preferred_image_format.present_mode);
    let extent = Self::choose_swap_extent(swapchain_support.capabilities, window_extent);

    let image_count =
      (swapchain_support.capabilities.min_image_count + 1).clamp(0, swapchain_support.capabilities.max_image_count);

    let graphics_family = device.get().graphics().family();
    let present_family = device.get().present().family();
    let queue_family_indices = &[graphics_family, present_family];

    let create_info = vk::SwapchainCreateInfoKHR::default()
      .surface(*surface.surface())
      .min_image_count(image_count)
      .image_format(surface_format.format)
      .image_color_space(surface_format.color_space)
      .present_mode(present_mode)
      .image_extent(extent)
      .image_array_layers(1)
      .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
      .image_sharing_mode(if graphics_family != present_family {
        vk::SharingMode::CONCURRENT
      } else {
        vk::SharingMode::EXCLUSIVE
      })
      .queue_family_indices(if graphics_family != present_family {
        queue_family_indices
      } else {
        Default::default()
      })
      .pre_transform(swapchain_support.capabilities.current_transform)
      .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
      .clipped(true);

    let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;

    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;

    Ok((swapchain, images, surface_format.format))
  }

  fn create_image_views(
    device: Handle<Device>,
    images: &[vk::Image],
    image_format: vk::Format,
  ) -> Result<Vec<vk::ImageView>, VulkanError> {
    let image_views = images
      .iter()
      .map(|&i| {
        let view_info = vk::ImageViewCreateInfo::default()
          .image(i)
          .view_type(vk::ImageViewType::TYPE_2D)
          .format(image_format)
          .subresource_range(
            vk::ImageSubresourceRange::default()
              .aspect_mask(vk::ImageAspectFlags::COLOR)
              .base_mip_level(0)
              .level_count(1)
              .base_array_layer(0)
              .layer_count(1),
          );

        unsafe { device.get().logical().create_image_view(&view_info, None) }
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(image_views)
  }

  fn choose_swap_surface_format(
    available_formats: Vec<vk::SurfaceFormatKHR>,
    preferred_color_space: ColorSpace,
  ) -> vk::SurfaceFormatKHR {
    for &format in &available_formats {
      match preferred_color_space {
        ColorSpace::Unorm => {
          if format.format == vk::Format::B8G8R8A8_UNORM && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
            return format;
          }
        }
        ColorSpace::SRGB => {
          if format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
            return format;
          }
        }
      }
    }

    available_formats
      .first()
      .cloned()
      .expect("no valid swap surfaces in vector")
  }

  fn choose_swap_present_mode(
    available_modes: Vec<vk::PresentModeKHR>,
    preferred_present_mode: PresentMode,
  ) -> vk::PresentModeKHR {
    for &mode in &available_modes {
      match preferred_present_mode {
        PresentMode::AutoAdaptive => {
          if mode == vk::PresentModeKHR::MAILBOX {
            return mode;
          }
        }
        PresentMode::AutoImmediate => {
          if mode == vk::PresentModeKHR::IMMEDIATE {
            return mode;
          }
        }
        PresentMode::AutoVsync => {
          if mode == vk::PresentModeKHR::FIFO {
            return mode;
          }
        }
      }
    }

    // fallback attempts
    for mode in available_modes {
      if let PresentMode::AutoAdaptive = preferred_present_mode {
        if mode == vk::PresentModeKHR::IMMEDIATE {
          return mode;
        }
      }
    }

    // ultimate fallback
    vk::PresentModeKHR::FIFO
  }

  fn choose_swap_extent(available_extents: vk::SurfaceCapabilitiesKHR, window_extent: vk::Extent2D) -> vk::Extent2D {
    if available_extents.current_extent.width != u32::MAX {
      available_extents.current_extent
    } else {
      vk::Extent2D::default()
        .width(window_extent.width.clamp(
          available_extents.min_image_extent.width,
          available_extents.max_image_extent.width,
        ))
        .height(window_extent.height.clamp(
          available_extents.min_image_extent.height,
          available_extents.max_image_extent.height,
        ))
    }
  }

  pub fn find_depth_format(device: Handle<Device>) -> vk::Format {
    let candidates = &[
      vk::Format::D32_SFLOAT,
      vk::Format::D32_SFLOAT_S8_UINT,
      vk::Format::D24_UNORM_S8_UINT,
    ];
    device.get().find_supported_format(
      candidates,
      vk::ImageTiling::OPTIMAL,
      vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
  }
}
