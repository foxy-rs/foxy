use anyhow::Result;
use ash::vk;
use foxy_types::handle::Handle;

pub mod builder;
pub mod config;
pub mod layout;

use self::{
  builder::{ConfigMissing, FragmentShaderMissing, RenderPipelineBuilder, VertexShaderMissing},
  config::{HasLayout, HasRenderPass, RenderPipelineConfig},
};
use crate::{
  device::Device,
  error::VulkanError,
  shader::{
    stage::{fragment::Fragment, vertex::Vertex},
    Shader,
  },
  unsupported_error,
};

pub struct RenderPipeline {
  device: Handle<Device>,
  pipeline: vk::Pipeline,
  config: RenderPipelineConfig<HasLayout, HasRenderPass>,
  vertex_shader: Handle<Shader<Vertex>>,
  fragment_shader: Handle<Shader<Fragment>>,
}

impl RenderPipeline {
  pub fn delete(&mut self) {
    unsafe {
      self.device.get().logical().destroy_pipeline(self.pipeline, None);
    }
  }
}

impl RenderPipeline {
  pub fn builder(
    device: Handle<Device>,
  ) -> RenderPipelineBuilder<VertexShaderMissing, FragmentShaderMissing, ConfigMissing> {
    RenderPipelineBuilder::new(device)
  }

  fn new(
    device: Handle<Device>,
    config: RenderPipelineConfig<HasLayout, HasRenderPass>,
    vertex_shader: Handle<Shader<Vertex>>,
    fragment_shader: Handle<Shader<Fragment>>,
  ) -> Result<Self> {
    let pipeline =
      Self::create_graphics_pipeline(device.clone(), vertex_shader.clone(), fragment_shader.clone(), &config)?;

    Ok(Self {
      device,
      pipeline,
      vertex_shader,
      fragment_shader,
      config,
    })
  }

  fn create_graphics_pipeline(
    device: Handle<Device>,
    vertex_shader: Handle<Shader<Vertex>>,
    fragment_shader: Handle<Shader<Fragment>>,
    config: &RenderPipelineConfig<HasLayout, HasRenderPass>,
  ) -> Result<vk::Pipeline, VulkanError> {
    let vertex_shader = vertex_shader.get();
    let fragment_shader = fragment_shader.get();
    // TODO: Overhaul pipeline creation to make it more type-driven

    let shader_stage_create_infos = &[vertex_shader.pipeline_info(), fragment_shader.pipeline_info()];
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly_info = config.input_assembly_info();
    let viewport_info = config.viewport_info();
    let rasterization_info = config.rasterization_info();
    let multisample_info = config.multisample_info();
    let depth_stencil_info = config.depth_stencil_info();
    let color_blend_info = config.color_blend_info();

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
      .stages(shader_stage_create_infos)
      .vertex_input_state(&vertex_input_info)
      .input_assembly_state(&input_assembly_info)
      .viewport_state(&viewport_info)
      .rasterization_state(&rasterization_info)
      .multisample_state(&multisample_info)
      .depth_stencil_state(&depth_stencil_info)
      .color_blend_state(&color_blend_info)
      .layout(config.layout().layout())
      .render_pass(config.render_pass())
      .subpass(config.subpass);

    unsafe {
      device
        .get()
        .logical()
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
        .map(|pipelines| pipelines[0])
        .map_err(|err| unsupported_error!("failed to create graphics pipelines: {err:?}"))
    }
  }
}
