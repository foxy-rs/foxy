use std::{mem::ManuallyDrop, ops::Deref, sync::Arc};

use foxy_vulkan::{
  builder::ValidationStatus,
  device::Device,
  error::VulkanError,
  image_format::ImageFormat,
  pipeline::{RenderPipeline, RenderPipelineConfig},
  swapchain::Swapchain,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::trace;

use self::render_data::RenderData;

pub mod command;
pub mod render_data;

pub struct Renderer {
  vulkan: ManuallyDrop<Arc<Device>>,
  swapchain: ManuallyDrop<Swapchain>,
  render_pipeline: ManuallyDrop<RenderPipeline>,

  render_data: RenderData,
}

impl Drop for Renderer {
  fn drop(&mut self) {
    trace!("Dropping Renderer");
    unsafe {
      trace!("> Dropping render pipeline");
      ManuallyDrop::drop(&mut self.render_pipeline);
      trace!("> Dropping swapchain");
      ManuallyDrop::drop(&mut self.swapchain);
      trace!("> Dropping vulkan");
      ManuallyDrop::drop(&mut self.vulkan);
    }
  }
}

impl Renderer {
  pub fn new(
    window: impl HasRawDisplayHandle + HasRawWindowHandle,
    window_size: (i32, i32),
  ) -> Result<Self, VulkanError> {
    let vulkan = ManuallyDrop::new(Arc::new(
      Device::builder()
        .with_window(&window)
        .with_validation(ValidationStatus::Enabled)
        .build()?,
    ));
    let swapchain = ManuallyDrop::new(Swapchain::new(vulkan.deref().clone(), window_size, ImageFormat {
      ..Default::default()
    })?);
    let render_pipeline = ManuallyDrop::new(
      RenderPipeline::builder(&vulkan)
        .with_vertex_shader("assets/shaders/simple/simple.vert")
        .with_fragment_shader("assets/shaders/simple/simple.frag")
        .with_config(RenderPipelineConfig::default())
        .build()?,
    );

    Ok(Self {
      vulkan,
      swapchain,
      render_pipeline,
      render_data: RenderData::default(),
    })
  }

  pub fn render(&mut self) -> Result<(), VulkanError> {
    Ok(())
  }

  pub fn update_render_data(&mut self, render_data: RenderData) -> Result<(), VulkanError> {
    Ok(())
  }
}
