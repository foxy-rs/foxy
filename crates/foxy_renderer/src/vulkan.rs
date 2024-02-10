#![deny(unsafe_op_in_unsafe_fn)]

use std::sync::Arc;

use foxy_utils::time::Time;
use tracing::*;
use vulkano::{
  command_buffer::{BlitImageInfo, ImageBlit, RenderingAttachmentInfo, RenderingInfo},
  format::ClearValue,
  image::{sampler::Filter, ImageAspects, ImageLayout, ImageSubresourceLayers},
  memory::allocator::StandardMemoryAllocator,
  render_pass::{AttachmentLoadOp, AttachmentStoreOp},
  swapchain::SwapchainPresentInfo,
  sync::{self, GpuFuture},
  Validated,
};
use winit::{event::WindowEvent, window::Window};

use self::{
  instance::FoxyInstance,
  surface::FoxySurface,
  types::frame_data::{FrameData, PrimaryCommandBufferBuilder},
};
use crate::{
  renderer::render_data::RenderData,
  vulkan::{
    device::FoxyDevice,
    error::VulkanError,
    swapchain::{image_format::PresentMode, FoxySwapchain},
  },
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
  allocator: Arc<StandardMemoryAllocator>,

  swapchain: FoxySwapchain,
  frame_index: usize,
  frame_data: Vec<FrameData>,

  is_dirty: bool,
  previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl Vulkan {
  pub fn new(window: Arc<Window>) -> Result<Self, VulkanError> {
    trace!("Initializing Vulkan");

    let instance = FoxyInstance::new(&window)?;
    let surface = FoxySurface::new(instance.vk().clone(), window.clone())?;
    let device = FoxyDevice::new(instance.vk().clone(), surface.vk().clone())?;
    let allocator = Arc::new(StandardMemoryAllocator::new_default(device.vk().clone()));

    let swapchain = FoxySwapchain::new(
      window.clone(),
      surface.vk().clone(),
      device.vk().clone(),
      allocator.clone(),
      PresentMode::AutoImmediate,
    )?;

    let frame_data = (0..FrameData::FRAME_OVERLAP)
      .map(|_| FrameData::new(&device))
      .collect::<Result<Vec<_>, VulkanError>>()?;

    let previous_frame_end = Some(sync::now(device.vk().clone()).boxed());

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
      previous_frame_end,
    })
  }

  pub fn resize(&mut self) {
    self.is_dirty = true;
  }

  pub fn render_frame(&mut self, _render_time: Time, _render_data: RenderData) -> Result<bool, VulkanError> {
    let image_extent: [u32; 2] = self.window.inner_size().into();

    if image_extent.contains(&0) {
      return Ok(false); // skip rendering when window is smol
    }

    self.previous_frame_end.as_mut().unwrap().cleanup_finished();

    if self.is_dirty {
      self.swapchain.rebuild()?;
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

    let mut cmd_builder = self.current_frame().primary_command(self.device.graphics_queue())?;

    let render_info = RenderingAttachmentInfo {
      load_op: AttachmentLoadOp::Clear,
      store_op: AttachmentStoreOp::Store,
      image_layout: ImageLayout::General,
      clear_value: Some(ClearValue::Float([0.2, 0.2, 0.2, 1.0])),
      ..RenderingAttachmentInfo::image_view(self.swapchain.draw_image_view())
    };

    cmd_builder
      .begin_rendering(RenderingInfo {
        color_attachments: vec![Some(render_info)],
        ..Default::default()
      })?
      .set_viewport(0, vec![self.swapchain.viewport().clone()].into())?;

    self.paint(&mut cmd_builder, image_index)?;

    cmd_builder.end_rendering()?;

    let region = ImageBlit {
      src_subresource: ImageSubresourceLayers {
        array_layers: 0..1,
        aspects: ImageAspects::COLOR,
        mip_level: 0,
      },
      src_offsets: [[0, 0, 0], self.swapchain.draw_image().extent()],
      dst_subresource: ImageSubresourceLayers {
        array_layers: 0..1,
        aspects: ImageAspects::COLOR,
        mip_level: 0,
      },
      dst_offsets: [[0, 0, 0], self.swapchain.image(image_index as usize).extent()],
      ..Default::default()
    };
    cmd_builder.blit_image(BlitImageInfo {
      src_image_layout: ImageLayout::TransferSrcOptimal,
      dst_image_layout: ImageLayout::TransferDstOptimal,
      regions: vec![region].into(),
      filter: Filter::Linear,
      ..BlitImageInfo::images(self.swapchain.draw_image(), self.swapchain.image(image_index as usize))
    })?;

    let cmd = cmd_builder.build()?;

    let future = self
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

    match future {
      Ok(future) => self.previous_frame_end = Some(future.boxed()),
      Err(Validated::Error(vulkano::VulkanError::OutOfDate)) => {
        self.is_dirty = true;
        self.previous_frame_end = Some(sync::now(self.device.vk().clone()).boxed());
      }
      Err(Validated::ValidationError(error)) => {
        error!("Validation error: `{error:?}`");
        self.previous_frame_end = Some(sync::now(self.device.vk().clone()).boxed());
      }
      Err(error) => {
        error!("failed to flush future: `{error:?}`");
        self.previous_frame_end = Some(sync::now(self.device.vk().clone()).boxed());
      }
    }

    self.frame_index = (self.frame_index + 1) % FrameData::FRAME_OVERLAP;

    Ok(false)
  }

  pub fn input(&mut self, _event: &WindowEvent) -> bool {
    false
  }
}

impl Vulkan {
  fn current_frame(&self) -> &FrameData {
    self.frame_data.get(self.frame_index).expect("invalid frame index")
  }

  fn paint(&self, cmd_builder: &mut PrimaryCommandBufferBuilder, image_index: u32) -> Result<(), VulkanError> {
    Ok(())
  }
}
