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
  pipeline::descriptor::DescriptorAllocator,
  shader::storage::ShaderStore,
  surface::Surface,
  swapchain::Swapchain,
  types::{allocated_image::AllocatedImage, frame_data::FrameData},
};
use crate::{
  error::RendererError,
  renderer::RenderBackend,
  vulkan::{
    pipeline::descriptor::{DescriptorLayoutBuilder, PoolSizeRatio},
    swapchain::image_format::{ColorSpace, ImageFormat, PresentMode},
  },
  vulkan_error,
};

pub mod device;
pub mod error;
pub mod image;
pub mod instance;
pub mod pipeline;
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

  draw_image_descriptor_layout: vk::DescriptorSetLayout,
  draw_image_descriptors: vk::DescriptorSet,
  global_descriptor_allocator: DescriptorAllocator,

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

    // init

    let instance = Instance::new(
      &window,
      if cfg!(debug_assertions) {
        ValidationStatus::Enabled
      } else {
        ValidationStatus::Disabled
      },
    )?;

    let surface = Surface::new(&window, &instance)?;
    let device = Device::new(&surface, instance.clone())?;

    let allocator_info = AllocatorCreateInfo::new(instance.raw(), device.logical(), *device.physical())
      .flags(vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS);
    let allocator = ManuallyDrop::new(Allocator::new(allocator_info).map_err(VulkanError::from)?);

    // init swapchain

    let swapchain = Swapchain::new(&instance, &surface, device.clone(), window_size, ImageFormat {
      color_space: ColorSpace::Unorm,
      present_mode: PresentMode::AutoImmediate,
    })?;

    let draw_extent = *vk::Extent3D::builder()
      .width(window_size.width as u32)
      .height(window_size.height as u32)
      .depth(1);
    let draw_image_format = vk::Format::R16G16B16A16_SFLOAT;

    let image_create_info = image::image_create_info(draw_extent, draw_image_format);
    let allocation_create_info = vk_mem::AllocationCreateInfo {
      usage: vk_mem::MemoryUsage::AutoPreferDevice,
      required_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
      ..Default::default()
    };

    let (image, allocation) =
      unsafe { allocator.create_image(&image_create_info, &allocation_create_info) }.map_err(VulkanError::from)?;

    let view_create_info = image::image_view_create_info(image, draw_image_format, vk::ImageAspectFlags::COLOR);
    let view = unsafe { device.logical().create_image_view(&view_create_info, None) }.map_err(VulkanError::from)?;

    let draw_image = AllocatedImage {
      image,
      view,
      allocation,
      extent: draw_extent,
      format: draw_image_format,
    };

    let frame_data = (0..FrameData::FRAME_OVERLAP)
      .map(|_| FrameData::new(&device))
      .collect::<Result<Vec<_>, VulkanError>>()?;

    // init descriptors

    let sizes = [PoolSizeRatio {
      kind: vk::DescriptorType::STORAGE_IMAGE,
      ratio: 1.0,
    }];

    let global_descriptor_allocator = DescriptorAllocator::new(device.clone(), 10, &sizes)?;
    let draw_image_descriptor_layout = DescriptorLayoutBuilder::new()
      .add_binding(0, vk::DescriptorType::STORAGE_IMAGE)
      .build(&device, vk::ShaderStageFlags::COMPUTE)?;
    let draw_image_descriptors = global_descriptor_allocator.allocate(draw_image_descriptor_layout)?;
    let image_info = *vk::DescriptorImageInfo::builder()
      .image_layout(vk::ImageLayout::GENERAL)
      .image_view(draw_image.view);

    let infos = &[image_info];
    let draw_image_write = *vk::WriteDescriptorSet::builder()
      .dst_binding(0)
      .dst_set(draw_image_descriptors)
      .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
      .image_info(infos);

    unsafe { device.logical().update_descriptor_sets(&[draw_image_write], &[]) };

    // init shaders

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
      draw_image_descriptor_layout,
      draw_image_descriptors,
      global_descriptor_allocator,
    })
  }

  fn delete(&mut self) {
    trace!("Waiting for GPU to finish...");
    let _ = unsafe { self.device.logical().device_wait_idle() }.log_error();

    trace!("Cleaning up Vulkan shaders...");

    self.shader_store.get_mut().delete();

    trace!("Cleaning up Vulkan handles/resources...");

    unsafe {
      self
        .device
        .logical()
        .destroy_descriptor_set_layout(self.draw_image_descriptor_layout, None)
    };
    self.global_descriptor_allocator.delete();

    self.draw_image.delete(&self.device, &self.allocator);

    for frame in &mut self.frame_data {
      frame.delete(&mut self.device);
    }

    self.swapchain.delete();

    unsafe { ManuallyDrop::drop(&mut self.allocator) };

    self.device.delete();
    self.surface.delete();
    self.instance.delete();
  }

  fn draw(&mut self, render_time: foxy_utils::time::Time) -> Result<(), RendererError> {
    match self.draw(render_time) {
      Ok(()) => Ok(()),
      Err(VulkanError::Suboptimal) => {
        // rebuild swapchain
        Ok(())
      }
      Err(VulkanError::Ash(vk::Result::ERROR_OUT_OF_DATE_KHR)) => {
        // rebuild swapchain
        Ok(())
      }
      Err(error) => Err(error)?,
    }
  }
}

impl Vulkan {
  fn draw(&mut self, render_time: foxy_utils::time::Time) -> Result<(), VulkanError> {
    let (image_index, is_suboptimal) = self.start_commands()?;
    if is_suboptimal {
      Err(VulkanError::Suboptimal)
    } else {
      self.draw_frame(image_index, &render_time)?;
      self.submit_commands(image_index)
    }
  }

  fn start_commands(&mut self) -> Result<(usize, bool), VulkanError> {
    let current_frame = self
      .frame_data
      .get_mut(self.frame_index)
      .ok_or_else(|| vulkan_error!("invalid frame"))?;
    let cmd = current_frame.master_command_buffer;

    let fences = &[current_frame.render_fence];
    unsafe {
      self
        .device
        .logical()
        .wait_for_fences(fences, true, Duration::from_secs(1).as_nanos() as u64)
    }?;
    unsafe { self.device.logical().reset_fences(fences) }?;

    match self.swapchain.acquire_next_image(current_frame.present_semaphore) {
      Ok((image_index, is_suboptimal)) => {
        if is_suboptimal {
          return Ok((image_index, is_suboptimal));
        }

        unsafe {
          self
            .device
            .logical()
            .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
        }?;

        let cmd_begin_info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.draw_extent = vk::Extent2D {
          width: self.draw_image.extent.width,
          height: self.draw_image.extent.height,
        };

        unsafe { self.device.logical().begin_command_buffer(cmd, &cmd_begin_info) }?;

        Ok((image_index, is_suboptimal))
      }
      Err(error) => {
        error!("{error}");
        Err(error)?
      }
    }
  }

  fn draw_background(&mut self, render_time: &Time) -> Result<(), VulkanError> {
    let current_frame = self
      .frame_data
      .get_mut(self.frame_index)
      .ok_or_else(|| vulkan_error!("invalid frame"))?;
    let cmd = current_frame.master_command_buffer;

    let time = render_time.since_start().as_secs_f32();
    let red_flash = (time / 0.25).sin().abs();
    let green_flash = (time / 1.25).sin().abs();
    let blue_flash = (time / 3.).sin().abs();
    let clear_value = vk::ClearColorValue {
      float32: [red_flash, green_flash, blue_flash, 1.0],
    };
    let clear_range = &[image::image_subresource_range(vk::ImageAspectFlags::COLOR)];

    unsafe {
      self.device.logical().cmd_clear_color_image(
        cmd,
        self.draw_image.image,
        vk::ImageLayout::GENERAL,
        &clear_value,
        clear_range,
      )
    }

    Ok(())
  }

  fn draw_frame(&mut self, image_index: usize, render_time: &Time) -> Result<(), VulkanError> {
    let current_frame = self
      .frame_data
      .get_mut(self.frame_index)
      .ok_or_else(|| vulkan_error!("invalid frame"))?;
    let cmd = current_frame.master_command_buffer;

    let swapchain_image = self
      .swapchain
      .image(image_index)
      .ok_or_else(|| vulkan_error!("invalid frame"))?;

    self
      .device
      .transition_image(cmd, self.draw_image.image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL);

    self.draw_background(render_time)?;

    self.device.transition_image(
      cmd,
      self.draw_image.image,
      vk::ImageLayout::GENERAL,
      vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
    );

    self.device.transition_image(
      cmd,
      swapchain_image,
      vk::ImageLayout::UNDEFINED,
      vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );

    self.device.copy_image(
      cmd,
      self.draw_image.image,
      swapchain_image,
      self.draw_extent,
      self.swapchain.extent(),
    );

    self.device.transition_image(
      cmd,
      swapchain_image,
      vk::ImageLayout::TRANSFER_DST_OPTIMAL,
      vk::ImageLayout::PRESENT_SRC_KHR,
    );

    Ok(())
  }

  fn submit_commands(&mut self, image_index: usize) -> Result<(), VulkanError> {
    let current_frame = self
      .frame_data
      .get_mut(self.frame_index)
      .ok_or_else(|| vulkan_error!("invalid frame"))?;
    let cmd = current_frame.master_command_buffer;

    unsafe { self.device.logical().end_command_buffer(cmd) }?;

    let cmd_infos = &[command_buffer_submit_info(cmd)];
    let wait_infos = &[semaphore_submit_info(
      current_frame.present_semaphore,
      vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT_KHR,
    )];
    let signal_infos = &[semaphore_submit_info(
      current_frame.render_semaphore,
      vk::PipelineStageFlags2::ALL_GRAPHICS,
    )];

    let submit = submit_info(cmd_infos, wait_infos, signal_infos);

    unsafe {
      self
        .device
        .logical()
        .queue_submit2(self.device.graphics().queue(), &[submit], current_frame.render_fence)
    }?;

    let swapchains = &[self.swapchain.khr()];
    let wait_semaphores = &[current_frame.render_semaphore];
    let image_indices = &[image_index as u32];
    let present_info = vk::PresentInfoKHR::builder()
      .swapchains(swapchains)
      .wait_semaphores(wait_semaphores)
      .image_indices(image_indices);

    let _is_suboptimal = self.swapchain.present(self.device.graphics().queue(), *present_info)?;

    self.frame_index = (self.frame_index + 1) % FrameData::FRAME_OVERLAP;

    Ok(())
  }
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
