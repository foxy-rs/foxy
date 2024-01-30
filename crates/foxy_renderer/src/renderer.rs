use foxy_utils::{
  time::Time,
  types::{handle::Handle, primitives::Dimensions},
};
use foxy_vulkan::{
  command_buffer::CommandBuffers,
  device::{builder::ValidationStatus, Device},
  error::VulkanError,
  image_format::ImageFormat,
  pipeline::{config::RenderPipelineConfig, layout::PipelineLayout, RenderPipeline, SimpleRenderPipeline},
  shader::set::ShaderSet,
  swapchain::Swapchain,
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::{error, trace};

use self::render_data::RenderData;

pub mod command;
pub mod render_data;

pub struct Renderer {
  command_buffers: CommandBuffers,
  render_pipeline_layout: PipelineLayout,
  render_pipeline: SimpleRenderPipeline,
  render_data: RenderData,

  swapchain: Handle<Swapchain>,

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

    let swapchain = Handle::new(Swapchain::new(device.clone(), window_size, ImageFormat {
      ..Default::default()
    })?);

    let render_pipeline_layout = PipelineLayout::new(device.clone())?;
    let config = RenderPipelineConfig::new(swapchain.get().size())?
      .with_render_pass(swapchain.get().render_pass())
      .with_layout(render_pipeline_layout.clone());
    let render_pipeline = SimpleRenderPipeline::new(
      device.clone(),
      config,
      ShaderSet::new(device.clone())
        .with_vertex("assets/shaders/simple/simple.vert")
        .with_fragment("assets/shaders/simple/simple.frag"),
    )?;

    let command_buffers = CommandBuffers::new(device.clone(), swapchain.clone())?;

    command_buffers.record(&render_pipeline)?;

    Ok(Self {
      device,
      swapchain,
      render_pipeline_layout,
      render_pipeline,
      render_data: RenderData::default(),
      command_buffers,
    })
  }

  pub fn delete(&mut self) {
    trace!("Deleting Renderer");
    trace!("> Deleting render_pipeline_layout");
    self.render_pipeline_layout.delete();
    trace!("> Deleting render_pipeline");
    self.render_pipeline.delete();
    trace!("> Deleting swapchain");
    self.swapchain.get_mut().delete();

    trace!("> Deleting device");
    self.device.get_mut().delete();
  }

  pub fn draw_frame(&mut self, _render_time: Time) -> Result<(), VulkanError> {
    // let result = self.swapchain.get_mut().next();
    // if let Some((image, is_optimal)) = result {
    //   if is_optimal {
    //     self.command_buffers.submit(image as usize)?;
    //   } else {
    //     error!("suboptimal!");
    //   }
    // }
    Ok(())
  }

  pub fn update_render_data(&mut self, render_data: RenderData) -> Result<(), VulkanError> {
    Ok(())
  }

  pub fn wait_for_gpu(&self) {
    self.device.get().wait_idle();
  }
}

/// Private Implemenation Details
impl Renderer {}
