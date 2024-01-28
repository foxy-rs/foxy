use self::render_data::RenderData;
use foxy_vulkan::{
  builder::ValidationStatus,
  error::VulkanError,
  pipeline::{RenderPipeline, RenderPipelineConfig},
  Vulkan,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub mod command;
pub mod render_data;

pub struct Renderer {
  render_pipeline: RenderPipeline,
  render_data: RenderData,
  vulkan: Vulkan,
}

impl Renderer {
  pub fn new(window: impl HasRawDisplayHandle + HasRawWindowHandle) -> Result<Self, VulkanError> {
    let vulkan = Vulkan::builder()
      .with_window(&window)
      .with_validation(ValidationStatus::Enabled)
      .build()?;
    let render_pipeline = RenderPipeline::builder()
      .with_vertex_shader(vulkan.shaders().get_vertex("assets/shaders/simple/simple.vert"))
      .with_fragment_shader(vulkan.shaders().get_fragment("assets/shaders/simple/simple.frag"))
      .with_config(RenderPipelineConfig::default())
      .build(&vulkan)?;

    Ok(Self {
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
