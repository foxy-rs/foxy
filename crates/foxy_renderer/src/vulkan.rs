#![deny(unsafe_op_in_unsafe_fn)]

use ash::vk;
use foxy_utils::{log::LogErr, types::handle::Handle};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::*;

use self::{
  device::Device,
  error::VulkanError,
  frame_data::FrameData,
  instance::Instance,
  shader::storage::ShaderStore,
  surface::Surface,
  swapchain::Swapchain,
};
use crate::{
  error::RendererError,
  renderer::RenderBackend,
  vulkan::swapchain::image_format::{ColorSpace, ImageFormat, PresentMode},
  vulkan_error,
};

pub mod device;
pub mod error;
pub mod frame_data;
pub mod instance;
pub mod queue;
pub mod shader;
pub mod surface;
pub mod swapchain;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ValidationStatus {
  Enabled,
  #[default]
  Disabled,
}

pub struct Vulkan {
  shader_store: Handle<ShaderStore>,

  frame_index: usize,
  frame_data: Vec<FrameData>,
  swapchain: Swapchain,

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

    let swapchain = Swapchain::new(&instance, &surface, device.clone(), window_size, ImageFormat {
      color_space: ColorSpace::Unorm,
      present_mode: PresentMode::AutoImmediate,
    })
    .map_err(|e| RendererError::Unrecoverable(e.into()))?;
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
      shader_store,
    })
  }

  fn delete(&mut self) {
    trace!("Waiting for GPU to finish...");
    let _ = unsafe { self.device.logical().device_wait_idle() }.log_error();

    trace!("Cleaning up Vulkan resources...");

    self.shader_store.get_mut().delete();

    for frame in &mut self.frame_data {
      frame.delete(&mut self.device);
    }
    self.swapchain.delete();

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
    unsafe { self.device.logical().wait_for_fences(fences, true, u64::MAX) }
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;
    unsafe { self.device.logical().reset_fences(fences) }.map_err(|e| RendererError::Unrecoverable(e.into()))?;

    let (image_index, is_suboptimal) = self
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

    let cmd_begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { self.device.logical().begin_command_buffer(cmd, &cmd_begin_info) }
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    Self::transition_image(
      &self.device,
      cmd,
      swapchain_image,
      vk::ImageLayout::UNDEFINED,
      vk::ImageLayout::GENERAL,
    );

    let time = render_time.since_start().as_secs_f32();
    let red_flash = (time / 80.).sin().abs();
    let green_flash = (time / 100.).sin().abs();
    let blue_flash = (time / 120.).sin().abs();
    let clear_value = vk::ClearColorValue {
      float32: [red_flash, green_flash, blue_flash, 1.0],
    };
    let clear_range = &[Self::image_subresource_range(vk::ImageAspectFlags::COLOR)];

    unsafe {
      self.device.logical().cmd_clear_color_image(
        cmd,
        swapchain_image,
        vk::ImageLayout::GENERAL,
        &clear_value,
        clear_range,
      )
    }

    Self::transition_image(
      &self.device,
      cmd,
      swapchain_image,
      vk::ImageLayout::GENERAL,
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
    let present_info = vk::PresentInfoKHR::default()
      .swapchains(swapchains)
      .wait_semaphores(wait_semaphores)
      .image_indices(image_indices);

    let _is_suboptimal = self
      .swapchain
      .present(self.device.graphics().queue(), present_info)
      .map_err(|e| RendererError::Unrecoverable(e.into()))?;

    self.frame_index = (self.frame_index + 1) % FrameData::FRAME_OVERLAP;

    Ok(())
  }
}

impl Vulkan {
  fn transition_image(
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

    let image_barrier = vk::ImageMemoryBarrier2::default()
      .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
      .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
      .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
      .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
      .old_layout(old_layout)
      .new_layout(new_layout)
      .subresource_range(Self::image_subresource_range(aspect_mask))
      .image(image);

    let image_barriers = &[image_barrier];
    let dependency_info = vk::DependencyInfo::default().image_memory_barriers(image_barriers);

    unsafe { device.logical().cmd_pipeline_barrier2(cmd, &dependency_info) };
  }

  fn image_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
      .aspect_mask(aspect_mask)
      .base_mip_level(0)
      .level_count(vk::REMAINING_MIP_LEVELS)
      .base_array_layer(0)
      .layer_count(vk::REMAINING_ARRAY_LAYERS)
  }

  fn semaphore_submit_info<'a>(
    semaphore: vk::Semaphore,
    stage_mask: vk::PipelineStageFlags2,
  ) -> vk::SemaphoreSubmitInfo<'a> {
    vk::SemaphoreSubmitInfo::default()
      .semaphore(semaphore)
      .stage_mask(stage_mask)
      .device_index(0)
      .value(1)
  }

  fn command_buffer_submit_info<'a>(command_buffer: vk::CommandBuffer) -> vk::CommandBufferSubmitInfo<'a> {
    vk::CommandBufferSubmitInfo::default()
      .command_buffer(command_buffer)
      .device_mask(0)
  }

  fn submit_info<'a>(
    command_buffer_infos: &'a [vk::CommandBufferSubmitInfo<'a>],
    wait_semaphore_infos: &'a [vk::SemaphoreSubmitInfo<'a>],
    signal_semaphore_infos: &'a [vk::SemaphoreSubmitInfo<'a>],
  ) -> vk::SubmitInfo2<'a> {
    vk::SubmitInfo2::default()
      .command_buffer_infos(command_buffer_infos)
      .wait_semaphore_infos(wait_semaphore_infos)
      .signal_semaphore_infos(signal_semaphore_infos)
  }
}
