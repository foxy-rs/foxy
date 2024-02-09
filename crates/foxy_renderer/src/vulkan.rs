#![deny(unsafe_op_in_unsafe_fn)]

use std::sync::Arc;

use foxy_utils::{log::LogErr, time::Time};
use tracing::*;
use vulkano::{
  command_buffer::{
    allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
    RenderingAttachmentInfo,
    RenderingInfo,
  },
  format::ClearValue,
  memory::allocator::StandardMemoryAllocator,
  render_pass::{AttachmentLoadOp, AttachmentStoreOp},
  swapchain::SwapchainPresentInfo,
  sync::{self, GpuFuture}, Validated,
};
use winit::{event::WindowEvent, window::Window};

use self::{instance::FoxyInstance, surface::FoxySurface, types::frame_data::FrameData};
use crate::{
  error::RendererError,
  renderer::render_data::RenderData,
  vulkan::{
    device::FoxyDevice,
    error::VulkanError,
    swapchain::{image_format::PresentMode, FoxySwapchain},
  },
  vulkan_error,
};

mod device;
pub mod error;
mod instance;
mod surface;
mod swapchain;
mod types;

pub struct Vulkan {
  window: Arc<Window>,
  instance: FoxyInstance,
  surface: FoxySurface,
  device: FoxyDevice,

  swapchain: FoxySwapchain,
  frame_index: usize,
  frame_data: Vec<FrameData>,

  allocator: Arc<StandardMemoryAllocator>,

  is_dirty: bool,
}

impl Vulkan {
  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    trace!("Initializing Vulkan");

    let instance = FoxyInstance::new(&window)?;
    let surface = FoxySurface::new(instance.vk().clone(), window.clone())?;
    let device = FoxyDevice::new(instance.vk().clone(), surface.vk().clone())?;

    let swapchain = FoxySwapchain::new(
      surface.vk().clone(),
      device.vk().clone(),
      window.inner_size(),
      PresentMode::AutoImmediate,
    )?;

    let frame_data = (0..FrameData::FRAME_OVERLAP)
      .map(|_| FrameData::new(&device))
      .collect::<Result<Vec<_>, VulkanError>>()?;

    let allocator = Arc::new(StandardMemoryAllocator::new_default(device.vk().clone()));

    Ok(Self {
      window,
      instance,
      surface,
      device,
      swapchain,
      frame_index: 0,
      frame_data,
      allocator,
      is_dirty: false,
    })
  }

  pub fn render(&mut self, render_time: Time, _render_data: RenderData) -> Result<bool, VulkanError> {
    let image_extent: [u32; 2] = self.window.inner_size().into();

    if image_extent.contains(&0) {
      return Ok(false); // skip rendering when window is smol
    }

    let current_frame = self
      .frame_data
      .get_mut(self.frame_index)
      .ok_or_else(|| vulkan_error!("invalid frame"))?;

    current_frame.previous_frame_end.as_mut().unwrap().cleanup_finished();

    if self.is_dirty {
      self.swapchain.rebuild(self.window.inner_size())?;
      self.is_dirty = false;
    }

    let (image_index, is_suboptimal, acquire_future) = match self.swapchain.acquire_next_image() {
      Ok(result) => result,
      Err(VulkanError::ValidatedVulkanoError(vulkano::Validated::Error(vulkano::VulkanError::OutOfDate))) => {
        self.is_dirty = true;
        return Ok(false);
      }
      Err(error) => Err(error)?,
    };

    if is_suboptimal {
      self.is_dirty = true;
    }

    let mut cmd_builder = current_frame.primary_command(self.device.graphics_queue())?;

    let render_info = Some(RenderingAttachmentInfo {
      load_op: AttachmentLoadOp::Clear,
      store_op: AttachmentStoreOp::Store,
      clear_value: Some(ClearValue::Float([0.2, 0.2, 0.2, 1.0])),
      ..RenderingAttachmentInfo::image_view(self.swapchain.image_view(image_index as usize).unwrap())
    });

    cmd_builder
      .begin_rendering(RenderingInfo {
        color_attachments: vec![render_info],
        ..Default::default()
      })?
      .end_rendering()?;

    let cmd = cmd_builder.build()?;

    let future = current_frame
      .previous_frame_end
      .take()
      .unwrap()
      .join(acquire_future)
      .then_execute(self.device.graphics_queue().clone(), cmd)?
      .then_swapchain_present(
        self.device.graphics_queue().clone(),
        SwapchainPresentInfo::swapchain_image_index(self.swapchain.vk().clone(), image_index),
      )
      .then_signal_fence_and_flush();

    match future.map_err(Validated::unwrap) {
        Ok(future) => current_frame.previous_frame_end = Some(future.boxed()),
        Err(vulkano::VulkanError::OutOfDate) => {
          self.is_dirty = true;
          current_frame.previous_frame_end = Some(sync::now(self.device.vk().clone()).boxed());
        },
        Err(error) => {
          error!("failed to flush future: `{error:?}`");
          current_frame.previous_frame_end = Some(sync::now(self.device.vk().clone()).boxed());
        },
    }

    self.frame_index = (self.frame_index + 1) % FrameData::FRAME_OVERLAP;

    Ok(false)
  }

  pub fn resize(&mut self) {
    self.is_dirty = true;
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }
}

impl Vulkan {}
