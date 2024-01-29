use foxy_types::{handle::Handle, primitives::Dimensions};
use foxy_util::time::Time;
use foxy_vulkan::{
  device::{builder::ValidationStatus, Device},
  error::VulkanError,
  image_format::ImageFormat,
  pipeline::{config::RenderPipelineConfig, layout::PipelineLayout, RenderPipeline},
  swapchain::Swapchain,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::trace;

use self::render_data::RenderData;

pub mod command;
pub mod render_data;

pub struct Renderer {
  render_pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,
  render_data: RenderData,

  swapchain: Swapchain,

  device: Handle<Device>,
}

impl Renderer {
  pub fn new(
    window: impl HasRawDisplayHandle + HasRawWindowHandle,
    window_size: Dimensions,
  ) -> Result<Self, VulkanError> {
    let device = Device::builder()
      .with_window(&window)
      .with_validation(ValidationStatus::Enabled)
      .build()?;

    let swapchain = Swapchain::new(device.clone(), window_size, ImageFormat { ..Default::default() })?;

    let render_pipeline_layout = PipelineLayout::new(device.clone())?;

    let render_pipeline = Self::create_render_pipeline(device.clone(), &swapchain, render_pipeline_layout.clone())?;

    let command_buffers = Self::create_command_buffers(device.clone(), &swapchain);

    Ok(Self {
      device,
      swapchain,
      render_pipeline_layout,
      render_pipeline,
      render_data: RenderData::default(),
    })
  }

  pub fn delete(&mut self) {
    trace!("Deleting Renderer");
    trace!("> Deleting render_pipeline_layout");
    self.render_pipeline_layout.delete();
    trace!("> Deleting render_pipeline");
    self.render_pipeline.delete();
    trace!("> Deleting swapchain");
    self.swapchain.delete();

    trace!("> Deleting device");
    self.device.get_mut().delete();
  }

  pub fn draw_frame(&mut self, _render_time: &Time) -> Result<(), VulkanError> {

    Ok(())
  }

  pub fn update_render_data(&mut self, render_data: RenderData) -> Result<(), VulkanError> {
    Ok(())
  }
}

/// Private Implemenation Details
impl Renderer {
  fn create_render_pipeline(
    device: Handle<Device>,
    swapchain: &Swapchain,
    layout: PipelineLayout,
  ) -> Result<RenderPipeline, VulkanError> {
    let config = RenderPipelineConfig::new(swapchain.size())?
      .with_render_pass(swapchain.render_pass())
      .with_layout(layout);

    let render_pipeline = RenderPipeline::builder(device)
      .with_vertex_shader("assets/shaders/simple/simple.vert")
      .with_fragment_shader("assets/shaders/simple/simple.frag")
      .with_config(config)
      .build()?;

    Ok(render_pipeline)
  }

  fn create_command_buffers(device: Handle<Device>, swapchain: &Swapchain) -> Result<(), VulkanError> {
    Ok(())
  }
}
