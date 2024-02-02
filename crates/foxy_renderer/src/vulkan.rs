#![deny(unsafe_op_in_unsafe_fn)]

use std::{mem::ManuallyDrop, time::Duration};

use ash::vk;
use foxy_utils::{log::LogErr, time::Time, types::handle::Handle};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::*;
use vk_mem::{Alloc, Allocator, AllocatorCreateInfo};

use self::{
  device::Device,
  error::VulkanError,
  instance::Instance,
  shader::storage::ShaderStore,
  surface::Surface,
  swapchain::Swapchain,
  types::{allocated_image::AllocatedImage, frame_data::FrameData},
};
use crate::{
  error::RendererError,
  renderer::RenderBackend,
  vulkan::swapchain::image_format::{ColorSpace, ImageFormat, PresentMode},
  vulkan_error,
};

pub mod device;
pub mod error;
pub mod instance;
pub mod shader;
pub mod surface;
pub mod swapchain;
pub mod types;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ValidationStatus {
  Enabled,
  #[default]
  Disabled,
}

pub struct Vulkan {
  shader_store: Handle<ShaderStore>,

  draw_image: AllocatedImage,
  draw_extent: vk::Extent2D,

  frame_index: usize,
  frame_data: Vec<FrameData>,
  swapchain: Swapchain,

  allocator: ManuallyDrop<Allocator>,

  device: Device,
  surface: Surface,
  instance: Instance,
}

impl RenderBackend for Vulkan {
  fn new(
    window: impl HasRawDisplayHandle + HasRawWindowHandle,
    window_size: foxy_utils::types::prelude::Dimensions,
  ) -> Result<Self, crate::error::RendererError>
  where
    Self: Sized,
  {
    trace!("Initializing Vulkan");

    let instance = Instance::new(
      &window,
      if cfg!(debug_assertions) {
        ValidationStatus::Enabled
      } else {
        ValidationStatus::Disabled
      },
    )
    .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let surface = Surface::new(&window, &instance).map_err(|e| RendererError::Unrecoverable(e.into()))?;
    let device = Device::new(&surface, instance.clone()).map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let allocator_info = AllocatorCreateInfo::new(instance.raw(), device.logical(), *device.physical())
      .flags(vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS);
    let allocator =
      ManuallyDrop::new(Allocator::new(allocator_info).map_err(|e| RendererError::Unrecoverable(e.into()))?);

    let swapchain = Swapchain::new(&instance, &surface, device.clone(), window_size, ImageFormat {
      color_space: ColorSpace::Unorm,
      present_mode: PresentMode::AutoImmediate,
    })
    .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let draw_extent = *vk::Extent3D::builder()
      .width(window_size.width as u32)
      .height(window_size.height as u32)
      .depth(1);
    let draw_image_format = vk::Format::R16G16B16A16_SFLOAT;

    let image_create_info = Self::image_create_info(draw_extent, draw_image_format);
    let allocation_create_info = vk_mem::AllocationCreateInfo {
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      required_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
      ..Default::default()
    };

    let (image, allocation) = unsafe { allocator.create_image(&image_create_info, &allocation_create_info) }
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let view_create_info = Self::image_view_create_info(image, draw_image_format, vk::ImageAspectFlags::COLOR);
    let view = unsafe { device.logical().create_image_view(&view_create_info, None) }
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let draw_image = AllocatedImage {
      image,
      view,
      allocation,
      extent: draw_extent,
      format: draw_image_format,
    };

    let frame_data = (0..FrameData::FRAME_OVERLAP)
      .map(|_| FrameData::new(&device))
      .collect::<Result<Vec<_>, VulkanError>>()
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let shader_store = Handle::new(ShaderStore::new(device.clone()));

    Ok(Self {
      instance,
      surface,
      device,
      swapchain,
      frame_data,
      frame_index: 0,
      allocator,
      shader_store,
      draw_image,
      draw_extent: Default::default(),
    })
  }

  fn delete(&mut self) {
    trace!("Waiting for GPU to finish...");
    let _ = unsafe { self.device.logical().device_wait_idle() }.log_error();

    trace!("Cleaning up Vulkan resources...");

    unsafe { self.device.logical().destroy_image_view(self.draw_image.view, None) };
    unsafe {
      self
        .allocator
        .destroy_image(self.draw_image.image, &mut self.draw_image.allocation)
    };

    self.shader_store.get_mut().delete();

    for frame in &mut self.frame_data {
      frame.delete(&mut self.device);
    }

    self.swapchain.delete();

    unsafe { ManuallyDrop::drop(&mut self.allocator) };

    self.device.delete();
    self.surface.delete();
    self.instance.delete();
  }

  fn draw(&mut self, render_time: foxy_utils::time::Time) -> Result<(), crate::error::RendererError> {
    let current_frame = self
      .frame_data
      .get_mut(self.frame_index)
      .ok_or_else(|| vulkan_error!("invalid frame"))
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let fences = &[current_frame.render_fence];
    unsafe {
      self
        .device
        .logical()
        .wait_for_fences(fences, true, Duration::from_secs(1).as_nanos() as u64)
    }
    .map_err(|e| RendererError::Unrecoverable(e.into()))?;
    unsafe { self.device.logical().reset_fences(fences) }.map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let (image_index, _is_suboptimal) = self
      .swapchain
      .acquire_next_image(current_frame.present_semaphore)
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;
    let swapchain_image = self
      .swapchain
      .image(image_index)
      .ok_or_else(|| vulkan_error!("invalid frame"))
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let cmd = current_frame.master_command_buffer;
    unsafe {
      self
        .device
        .logical()
        .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
    }
    .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let cmd_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    self.draw_extent = vk::Extent2D {
      width: self.draw_image.extent.width,
      height: self.draw_image.extent.height,
    };

    unsafe { self.device.logical().begin_command_buffer(cmd, &cmd_begin_info) }
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    Self::transition_image(
      &self.device,
      cmd,
      self.draw_image.image,
      vk::ImageLayout::UNDEFINED,
      vk::ImageLayout::GENERAL,
    );

    Self::draw_background(&self.device, cmd, &self.draw_image, &render_time)
      .map_err(|e| RendererError::Recoverable(e.into()))?;

    Self::transition_image(
      &self.device,
      cmd,
      self.draw_image.image,
      vk::ImageLayout::GENERAL,
      vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
    );

    Self::transition_image(
      &self.device,
      cmd,
      swapchain_image,
      vk::ImageLayout::UNDEFINED,
      vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );

    Self::copy_image_to_image(
      &self.device,
      cmd,
      self.draw_image.image,
      swapchain_image,
      self.draw_extent,
      self.swapchain.extent(),
    );

    Self::transition_image(
      &self.device,
      cmd,
      swapchain_image,
      vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      vk::ImageLayout::PRESENT_SRC_KHR,
    );

    unsafe { self.device.logical().end_command_buffer(cmd) }.map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let cmd_infos = &[Self::command_buffer_submit_info(cmd)];
    let wait_infos = &[Self::semaphore_submit_info(
      current_frame.present_semaphore,
      vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT_KHR,
    )];
    let signal_infos = &[Self::semaphore_submit_info(
      current_frame.render_semaphore,
      vk::PipelineStageFlags2::ALL_GRAPHICS,
    )];

    let submit = Self::submit_info(cmd_infos, wait_infos, signal_infos);

    unsafe {
      self
        .device
        .logical()
        .queue_submit2(self.device.graphics().queue(), &[submit], current_frame.render_fence)
    }
    .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let swapchains = &[self.swapchain.khr()];
    let wait_semaphores = &[current_frame.render_semaphore];
    let image_indices = &[image_index as u32];
    let present_info = vk::PresentInfoKHR::builder()
      .swapchains(swapchains)
      .wait_semaphores(wait_semaphores)
      .image_indices(image_indices);

    let _is_suboptimal = self
      .swapchain
      .present(self.device.graphics().queue(), *present_info)
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    self.frame_index = (self.frame_index + 1) % FrameData::FRAME_OVERLAP;

    Ok(())
  }
}

impl Vulkan {
  pub fn draw_background(
    device: &Device,
    cmd: vk::CommandBuffer,
    image: &AllocatedImage,
    render_time: &Time,
  ) -> Result<(), VulkanError> {
    let time = render_time.since_start().as_secs_f32();
    let red_flash = (time / 1.).sin().abs();
    let green_flash = (time / 2.).sin().abs();
    let blue_flash = (time / 3.).sin().abs();
    let clear_value = vk::ClearColorValue {
      float32: [red_flash, green_flash, blue_flash, 1.0],
    };
    let clear_range = &[Self::image_subresource_range(vk::ImageAspectFlags::COLOR)];

    unsafe {
      device
        .logical()
        .cmd_clear_color_image(cmd, image.image, vk::ImageLayout::GENERAL, &clear_value, clear_range)
    }

    Ok(())
  }

  pub fn transition_image(
    device: &Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
  ) {
    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {
      vk::ImageAspectFlags::DEPTH
    } else {
      vk::ImageAspectFlags::COLOR
    };

    let image_barrier = vk::ImageMemoryBarrier2::builder()
      .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
      .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
      .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
      .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
      .old_layout(old_layout)
      .new_layout(new_layout)
      .subresource_range(Self::image_subresource_range(aspect_mask))
      .image(image);

    let image_barriers = &[*image_barrier];
    let dependency_info = vk::DependencyInfo::builder().image_memory_barriers(image_barriers);

    unsafe { device.logical().cmd_pipeline_barrier2(cmd, &dependency_info) };
  }

  pub fn image_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    *vk::ImageSubresourceRange::builder()
      .aspect_mask(aspect_mask)
      .base_mip_level(0)
      .level_count(vk::REMAINING_MIP_LEVELS)
      .base_array_layer(0)
      .layer_count(vk::REMAINING_ARRAY_LAYERS)
  }

  pub fn semaphore_submit_info<'a>(
    semaphore: vk::Semaphore,
    stage_mask: vk::PipelineStageFlags2,
  ) -> vk::SemaphoreSubmitInfo {
    *vk::SemaphoreSubmitInfo::builder()
      .semaphore(semaphore)
      .stage_mask(stage_mask)
      .device_index(0)
      .value(1)
  }

  pub fn command_buffer_submit_info<'a>(command_buffer: vk::CommandBuffer) -> vk::CommandBufferSubmitInfo {
    *vk::CommandBufferSubmitInfo::builder()
      .command_buffer(command_buffer)
      .device_mask(0)
  }

  pub fn submit_info<'a>(
    command_buffer_infos: &'a [vk::CommandBufferSubmitInfo],
    wait_semaphore_infos: &'a [vk::SemaphoreSubmitInfo],
    signal_semaphore_infos: &'a [vk::SemaphoreSubmitInfo],
  ) -> vk::SubmitInfo2 {
    *vk::SubmitInfo2::builder()
      .command_buffer_infos(command_buffer_infos)
      .wait_semaphore_infos(wait_semaphore_infos)
      .signal_semaphore_infos(signal_semaphore_infos)
  }

  pub fn image_create_info(extent: vk::Extent3D, format: vk::Format) -> vk::ImageCreateInfo {
    *vk::ImageCreateInfo::builder()
      .image_type(vk::ImageType::TYPE_2D)
      .format(format)
      .extent(extent)
      .mip_levels(1)
      .array_layers(1)
      .samples(vk::SampleCountFlags::TYPE_1)
      .tiling(vk::ImageTiling::OPTIMAL)
      .usage(
        vk::ImageUsageFlags::TRANSFER_SRC
          | vk::ImageUsageFlags::TRANSFER_DST
          | vk::ImageUsageFlags::STORAGE
          | vk::ImageUsageFlags::COLOR_ATTACHMENT,
      )
  }

  pub fn image_view_create_info(
    image: vk::Image,
    format: vk::Format,
    mask: vk::ImageAspectFlags,
  ) -> vk::ImageViewCreateInfo {
    let subresource = vk::ImageSubresourceRange::builder()
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1)
      .aspect_mask(mask);

    *vk::ImageViewCreateInfo::builder()
      .view_type(vk::ImageViewType::TYPE_2D)
      .image(image)
      .format(format)
      .subresource_range(*subresource)
  }

  pub fn copy_image_to_image(
    device: &Device,
    cmd: vk::CommandBuffer,
    source: vk::Image,
    dest: vk::Image,
    source_size: vk::Extent2D,
    dest_size: vk::Extent2D,
  ) {
    let src_subres = vk::ImageSubresourceLayers::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_array_layer(0)
      .layer_count(1)
      .mip_level(0);
    let dst_subres = vk::ImageSubresourceLayers::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_array_layer(0)
      .layer_count(1)
      .mip_level(0);

    let blit_region = vk::ImageBlit2::builder()
      .src_offsets([vk::Offset3D::default(), vk::Offset3D {
        x: source_size.width as i32,
        y: source_size.height as i32,
        z: 1,
      }])
      .dst_offsets([vk::Offset3D::default(), vk::Offset3D {
        x: dest_size.width as i32,
        y: dest_size.height as i32,
        z: 1,
      }])
      .src_subresource(*src_subres)
      .dst_subresource(*dst_subres);

    let regions = &[*blit_region];
    let blit_info = vk::BlitImageInfo2::builder()
      .dst_image(dest)
      .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
      .src_image(source)
      .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
      .filter(vk::Filter::LINEAR)
      .regions(regions);

    unsafe { device.logical().cmd_blit_image2(cmd, &blit_info) };
  }
}
