use foxy_vulkan::{
  builder::ValidationStatus,
  error::VulkanError,
  pipeline::{RenderPipeline, RenderPipelineConfig},
  swapchain::Swapchain,
  Vulkan,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use self::render_data::RenderData;

pub mod command;
pub mod render_data;

pub struct Renderer {
  render_data: RenderData,

  render_pipeline: RenderPipeline,
  swapchain: Swapchain,
  vulkan: Vulkan,
}

impl Renderer {
  pub fn new(
    window: impl HasRawDisplayHandle + HasRawWindowHandle,
    window_size: (i32, i32),
  ) -> Result<Self, VulkanError> {
    let vulkan = Vulkan::builder()
      .with_window(&window)
      .with_validation(ValidationStatus::Enabled)
      .build()?;
    let swapchain = Swapchain::new(&vulkan)?;
    let render_pipeline = RenderPipeline::builder()
      .with_vertex_shader(vulkan.shaders().get_vertex("assets/shaders/simple/simple.vert"))
      .with_fragment_shader(vulkan.shaders().get_fragment("assets/shaders/simple/simple.frag"))
      .with_config(RenderPipelineConfig::default())
      .build(&vulkan)?;

    Ok(Self {
      swapchain,
      render_pipeline,
      render_data: RenderData::default(),
      vulkan,
    })
  }

  pub fn render(&mut self) -> Result<(), VulkanError> {
    Ok(())
  }

  pub fn update_render_data(&mut self, render_data: RenderData) -> Result<(), VulkanError> {
    Ok(())
  }
}
