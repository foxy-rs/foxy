use std::{mem::ManuallyDrop, ops::Deref, sync::Arc};

use ash::{extensions::khr, vk};
use foxy_util::log::LogErr;
use itertools::Itertools;

use crate::{error::VulkanError, image::Image, Vulkan};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ImageFormat {
  pub present_mode: PresentMode,
  pub color_space: ColorSpace,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
  Unorm,
  #[default]
  SRGB,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
  AutoAdaptive,
  AutoImmediate,
  #[default]
  AutoVsync,
}

pub struct Swapchain {
  vulkan: Arc<Vulkan>,

  current_frame_index: usize,
  images_in_flight: ManuallyDrop<Vec<vk::Fence>>,
  fences_in_flight: ManuallyDrop<Vec<vk::Fence>>,
  image_avaiable_semaphores: ManuallyDrop<Vec<vk::Semaphore>>,
  render_finished_semaphores: ManuallyDrop<Vec<vk::Semaphore>>,

  image_views: ManuallyDrop<Vec<vk::ImageView>>,
  images: Vec<vk::Image>,

  depth_image_views: ManuallyDrop<Vec<vk::ImageView>>,
  depth_images: ManuallyDrop<Vec<Image>>,

  render_pass: ManuallyDrop<vk::RenderPass>,
  framebuffers: ManuallyDrop<Vec<vk::Framebuffer>>,

  extent: vk::Extent2D,
  image_format: vk::Format,

  swapchain: ManuallyDrop<vk::SwapchainKHR>,
  swapchain_loader: khr::Swapchain,
}

impl Drop for Swapchain {
  fn drop(&mut self) {
    unsafe {
      // image views
      for &image_view in self.image_views.iter() {
        self.vulkan.logical().destroy_image_view(image_view, None);
      }
      self.image_views.clear();
      ManuallyDrop::drop(&mut self.image_views);

      // swapchain
      self.swapchain_loader.destroy_swapchain(*self.swapchain, None);
      ManuallyDrop::drop(&mut self.swapchain);

      // depth images
      for &image_view in self.depth_image_views.iter() {
        self.vulkan.logical().destroy_image_view(image_view, None);
      }
      self.depth_image_views.clear();
      ManuallyDrop::drop(&mut self.depth_image_views);
      self.depth_images.clear();
      ManuallyDrop::drop(&mut self.depth_images);

      // framebuffer
      for &framebuffer in self.framebuffers.iter() {
        self.vulkan.logical().destroy_framebuffer(framebuffer, None);
      }
      self.framebuffers.clear();
      ManuallyDrop::drop(&mut self.framebuffers);

      // render pass
      self.vulkan.logical().destroy_render_pass(*self.render_pass, None);
      ManuallyDrop::drop(&mut self.render_pass);

      // sync objects
      for i in 0..Self::MAX_FRAMES_IN_FLIGHT {
        self
          .vulkan
          .logical()
          .destroy_semaphore(self.render_finished_semaphores[i], None);
        self
          .vulkan
          .logical()
          .destroy_semaphore(self.image_avaiable_semaphores[i], None);
        self.vulkan.logical().destroy_fence(self.fences_in_flight[i], None);
        self.vulkan.logical().destroy_fence(self.images_in_flight[i], None);
      }
      ManuallyDrop::drop(&mut self.render_finished_semaphores);
      ManuallyDrop::drop(&mut self.image_avaiable_semaphores);
      ManuallyDrop::drop(&mut self.fences_in_flight);
      ManuallyDrop::drop(&mut self.images_in_flight);
    }
  }
}

impl Iterator for Swapchain {
  type Item = (u32, bool);

  fn next(&mut self) -> Option<Self::Item> {
    self.acquire_next_image().log_error().ok()
  }
}

impl Swapchain {
  const MAX_FRAMES_IN_FLIGHT: usize = 2;

  pub fn new(
    vulkan: Arc<Vulkan>,
    window_size: (i32, i32),
    preferred_image_format: ImageFormat,
  ) -> Result<Self, VulkanError> {
    let extent = vk::Extent2D::default()
      .width(window_size.0 as u32)
      .height(window_size.0 as u32);

    let swapchain_loader = khr::Swapchain::new(vulkan.instance(), &vulkan.logical());
    let (swapchain, images, image_format) =
      Self::create_swap_chain(&swapchain_loader, vulkan.clone(), preferred_image_format, extent)?;
    let swapchain = ManuallyDrop::new(swapchain);

    let image_views = ManuallyDrop::new(Self::create_image_views(vulkan.logical(), &images, image_format)?);

    let render_pass = ManuallyDrop::new(Self::create_render_pass(vulkan.clone(), image_format)?);

    let (depth_image_views, depth_images) = Self::create_depth_resources(vulkan.clone(), extent, &images)?;
    let depth_image_views = ManuallyDrop::new(depth_image_views);
    let depth_images = ManuallyDrop::new(depth_images);

    let framebuffers = ManuallyDrop::new(Self::create_framebuffers(
      vulkan.logical(),
      extent,
      *render_pass,
      &image_views,
      &depth_image_views,
    )?);

    let (image_avaiable_semaphores, render_finished_semaphores, fences_in_flight, images_in_flight) =
      Self::create_sync_objects(vulkan.clone(), &images)?;
    let image_avaiable_semaphores = ManuallyDrop::new(image_avaiable_semaphores);
    let render_finished_semaphores = ManuallyDrop::new(render_finished_semaphores);
    let fences_in_flight = ManuallyDrop::new(fences_in_flight);
    let images_in_flight = ManuallyDrop::new(images_in_flight);

    Ok(Self {
      vulkan,
      swapchain,
      current_frame_index: 0,
      images_in_flight,
      fences_in_flight,
      image_avaiable_semaphores,
      render_finished_semaphores,
      image_views,
      images,
      depth_image_views,
      depth_images,
      render_pass,
      framebuffers,
      extent,
      image_format,
      swapchain_loader,
    })
  }

  fn current_fence_in_flight(&self) -> vk::Fence {
    self.fences_in_flight[self.current_frame_index]
  }

  fn current_image_available_semaphore(&self) -> vk::Semaphore {
    self.image_avaiable_semaphores[self.current_frame_index]
  }

  fn current_render_finished_semaphore(&self) -> vk::Semaphore {
    self.render_finished_semaphores[self.current_frame_index]
  }

  fn acquire_next_image(&mut self) -> Result<(u32, bool), VulkanError> {
    unsafe {
      self
        .vulkan
        .logical()
        .wait_for_fences(&[self.current_fence_in_flight()], true, u64::MAX)
    }?;

    let result = unsafe {
      self.swapchain_loader.acquire_next_image(
        *self.swapchain,
        u64::MAX,
        self.current_image_available_semaphore(),
        vk::Fence::null(),
      )
    }?;

    Ok(result)
  }

  pub fn submit_command_buffers(
    &mut self,
    buffers: &[vk::CommandBuffer],
    image_index: usize,
  ) -> Result<bool, VulkanError> {
    if self.images_in_flight[image_index] != vk::Fence::null() {
      unsafe {
        self
          .vulkan
          .logical()
          .wait_for_fences(&[self.images_in_flight[image_index]], true, u64::MAX)
      }?;
    }

    self.images_in_flight[image_index] = self.images_in_flight[self.current_frame_index];

    let wait_semaphores = &[self.current_image_available_semaphore()];
    let signal_semaphores = &[self.current_render_finished_semaphore()];
    let submit_info = vk::SubmitInfo::default()
      .command_buffers(buffers)
      .wait_semaphores(wait_semaphores)
      .signal_semaphores(signal_semaphores);

    unsafe { self.vulkan.logical().reset_fences(&[self.current_fence_in_flight()]) }?;

    unsafe {
      self
        .vulkan
        .logical()
        .queue_submit(*self.vulkan.graphics_queue(), &[submit_info], self.current_fence_in_flight())
    }?;

    let swapchains = &[*self.swapchain];
    let image_indices = &[image_index as u32];
    let present_info = vk::PresentInfoKHR::default()
      .wait_semaphores(signal_semaphores)
      .swapchains(swapchains)
      .image_indices(image_indices);

    let result = unsafe {
      self
        .swapchain_loader
        .queue_present(*self.vulkan.present_queue(), &present_info)
    };

    self.current_frame_index = (self.current_frame_index + 1) % Self::MAX_FRAMES_IN_FLIGHT;

    result.map_err(VulkanError::from)
  }

  pub fn extent_aspect_ratio(&self) -> f32 {
    self.width() as f32 / self.height() as f32
  }

  pub fn width(&self) -> u32 {
    self.extent.width
  }

  pub fn height(&self) -> u32 {
    self.extent.height
  }

  pub fn image_format(&self) -> vk::Format {
    self.image_format
  }

  pub fn image_count(&self) -> usize {
    self.images.len()
  }

  pub fn image_view(&self, index: usize) -> vk::ImageView {
    self.image_views[index]
  }

  pub fn render_pass(&self) -> vk::RenderPass {
    *self.render_pass
  }

  pub fn frame_buffer(&self, index: usize) -> vk::Framebuffer {
    self.framebuffers[index]
  }

  fn create_swap_chain(
    swapchain_loader: &khr::Swapchain,
    vulkan: Arc<Vulkan>,
    preferred_image_format: ImageFormat,
    window_extent: vk::Extent2D,
  ) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format), VulkanError> {
    let swapchain_support = vulkan.swapchain_support()?;

    let surface_format =
      Self::choose_swap_surface_format(swapchain_support.formats, preferred_image_format.color_space);
    let present_mode =
      Self::choose_swap_present_mode(swapchain_support.present_modes, preferred_image_format.present_mode);
    let extent = Self::choose_swap_extent(swapchain_support.capabilities, window_extent);

    let image_count =
      (swapchain_support.capabilities.min_image_count + 1).clamp(0, swapchain_support.capabilities.max_image_count);

    let indices = vulkan.queue_families()?;
    let queue_family_indices = &[indices.graphics_family, indices.present_family];

    let create_info = vk::SwapchainCreateInfoKHR::default()
      .surface(*vulkan.surface().surface())
      .surface(*vulkan.surface().surface())
      .min_image_count(image_count)
      .image_format(surface_format.format)
      .image_color_space(surface_format.color_space)
      .present_mode(present_mode)
      .image_extent(extent)
      .image_array_layers(1)
      .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
      .image_sharing_mode(if indices.graphics_family != indices.present_family {
        vk::SharingMode::CONCURRENT
      } else {
        vk::SharingMode::EXCLUSIVE
      })
      .queue_family_indices(if indices.graphics_family != indices.present_family {
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
    device: Arc<ash::Device>,
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

        unsafe { device.create_image_view(&view_info, None) }
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(image_views)
  }

  fn create_render_pass(
    vulkan: Arc<Vulkan>,
    swapchain_image_format: vk::Format,
  ) -> Result<vk::RenderPass, VulkanError> {
    let depth_attachment = vk::AttachmentDescription::default()
      .format(Self::find_depth_format(vulkan.clone()))
      .samples(vk::SampleCountFlags::TYPE_1)
      .load_op(vk::AttachmentLoadOp::CLEAR)
      .store_op(vk::AttachmentStoreOp::DONT_CARE)
      .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let depth_attachment_ref = vk::AttachmentReference::default()
      .attachment(1)
      .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let color_attachment = vk::AttachmentDescription::default()
      .format(swapchain_image_format)
      .samples(vk::SampleCountFlags::TYPE_1)
      .load_op(vk::AttachmentLoadOp::CLEAR)
      .store_op(vk::AttachmentStoreOp::DONT_CARE)
      .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      .initial_layout(vk::ImageLayout::UNDEFINED)
      .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_refs = &[vk::AttachmentReference::default()
      .attachment(1)
      .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

    let subpasses = &[vk::SubpassDescription::default()
      .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
      .color_attachments(color_attachment_refs)
      .depth_stencil_attachment(&depth_attachment_ref)];

    let dependencies = &[vk::SubpassDependency::default()
      .src_subpass(vk::SUBPASS_EXTERNAL)
      .src_access_mask(vk::AccessFlags::empty())
      .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
      .dst_subpass(0)
      .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
      .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)];

    let attachments = &[color_attachment, depth_attachment];

    let render_pass_info = vk::RenderPassCreateInfo::default()
      .attachments(attachments)
      .subpasses(subpasses)
      .dependencies(dependencies);

    unsafe { vulkan.logical().create_render_pass(&render_pass_info, None) }.map_err(VulkanError::from)
  }

  fn create_framebuffers(
    device: Arc<ash::Device>,
    swapchain_extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    swapchain_image_views: &[vk::ImageView],
    depth_image_views: &[vk::ImageView],
  ) -> Result<Vec<vk::Framebuffer>, VulkanError> {
    let framebuffers = (0..swapchain_image_views.len())
      .map(|i| {
        let attachments = &[swapchain_image_views[i], depth_image_views[i]];

        let framebuffer_info = vk::FramebufferCreateInfo::default()
          .render_pass(render_pass)
          .attachments(attachments)
          .width(swapchain_extent.width)
          .height(swapchain_extent.height)
          .layers(1);

        unsafe { device.create_framebuffer(&framebuffer_info, None) }
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(framebuffers)
  }

  fn create_depth_resources(
    vulkan: Arc<Vulkan>,
    swapchain_extent: vk::Extent2D,
    images: &[vk::Image],
  ) -> Result<(Vec<vk::ImageView>, Vec<Image>), VulkanError> {
    let depth_format = Self::find_depth_format(vulkan.clone());

    let (views, images) = images
      .iter()
      .zip(images.iter())
      .map(|_| {
        let image_info = vk::ImageCreateInfo::default()
          .image_type(vk::ImageType::TYPE_2D)
          .extent(
            vk::Extent3D::default()
              .width(swapchain_extent.width)
              .height(swapchain_extent.height)
              .depth(1),
          )
          .format(depth_format)
          .mip_levels(1)
          .array_layers(1)
          .initial_layout(vk::ImageLayout::UNDEFINED)
          .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
          .samples(vk::SampleCountFlags::TYPE_1)
          .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = Image::new(vulkan.clone(), image_info, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;

        let view_info = vk::ImageViewCreateInfo::default()
          .image(image.image())
          .view_type(vk::ImageViewType::TYPE_2D)
          .format(depth_format)
          .subresource_range(
            vk::ImageSubresourceRange::default()
              .aspect_mask(vk::ImageAspectFlags::DEPTH)
              .base_mip_level(0)
              .level_count(1)
              .base_array_layer(0)
              .layer_count(1),
          );

        let view = unsafe { vulkan.logical().create_image_view(&view_info, None) }?;

        Ok((view, image))
      })
      .collect::<Result<Vec<(vk::ImageView, Image)>, VulkanError>>()?
      .into_iter()
      .unzip();

    Ok((views, images))
  }

  fn create_sync_objects(
    vulkan: Arc<Vulkan>,
    images: &[vk::Image],
  ) -> Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>, Vec<vk::Fence>), VulkanError> {
    let sema_info = vk::SemaphoreCreateInfo::default();
    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

    let mut image_semaphores = vec![Default::default(); Self::MAX_FRAMES_IN_FLIGHT];
    for fence in &mut image_semaphores {
      *fence = unsafe { vulkan.logical().create_semaphore(&sema_info, None) }?;
    }

    let mut render_semaphores = vec![Default::default(); Self::MAX_FRAMES_IN_FLIGHT];
    for fence in &mut render_semaphores {
      *fence = unsafe { vulkan.logical().create_semaphore(&sema_info, None) }?;
    }

    let mut flight_fences = vec![Default::default(); Self::MAX_FRAMES_IN_FLIGHT];
    for fence in &mut flight_fences {
      *fence = unsafe { vulkan.logical().create_fence(&fence_info, None) }?;
    }

    let mut image_fences = vec![Default::default(); Self::MAX_FRAMES_IN_FLIGHT];

    Ok((image_semaphores, render_semaphores, flight_fences, image_fences))
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

    return available_formats[0];
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
      match preferred_present_mode {
        PresentMode::AutoAdaptive => {
          if mode == vk::PresentModeKHR::IMMEDIATE {
            return mode;
          }
        }
        _ => {}
      }
    }

    // ultimate fallback
    return vk::PresentModeKHR::FIFO;
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

  pub fn find_depth_format(vulkan: Arc<Vulkan>) -> vk::Format {
    let candidates = &[
      vk::Format::D32_SFLOAT,
      vk::Format::D32_SFLOAT_S8_UINT,
      vk::Format::D24_UNORM_S8_UINT,
    ];
    vulkan.find_supported_format(
      candidates,
      vk::ImageTiling::OPTIMAL,
      vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
  }
}
