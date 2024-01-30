use foxy_utils::{
  time::Time,
  types::{handle::Handle, primitives::Dimensions},
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::*;

use self::render_data::RenderData;
use crate::vulkan::{
  builder::ValidationStatus,
  error::VulkanError,
  Vulkan,
};

pub mod command;
pub mod render_data;

pub struct Renderer {
  // command_buffers: CommandBuffers,
  // render_pipeline_layout: PipelineLayout,
  // render_pipeline: SimpleRenderPipeline,
  // render_data: RenderData,

  // swapchain: Handle<Swapchain>,
  vulkan: Handle<Vulkan>,
}

impl Renderer {
  pub fn new(
    window: impl HasRawDisplayHandle + HasRawWindowHandle,
    window_size: Dimensions,
  ) -> Result<Self, VulkanError> {
    let device = Vulkan::builder()
      .with_window(&window, window_size)
      .with_validation(ValidationStatus::Enabled)
      .build()?;

    Ok(Self { vulkan: device })
  }

  pub fn delete(&mut self) {
    self.vulkan.get_mut().delete();
  }

  pub fn draw_frame(&mut self, _render_time: Time) -> Result<(), VulkanError> {
    Ok(())
  }

  pub fn update_render_data(&mut self, _render_data: RenderData) -> Result<(), VulkanError> {
    Ok(())
  }
}

/// Private Implemenation Details
impl Renderer {}
