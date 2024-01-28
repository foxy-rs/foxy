use anyhow::{Context, Result};
use ash::vk;
use std::sync::Arc;

use crate::{
  error::VulkanError,
  shader::{
    stage::{fragment::Fragment, vertex::Vertex},
    Shader,
  },
  unsupported_error, Vulkan,
};

pub struct RenderPipeline {
  device: Arc<ash::Device>,
  pipeline: vk::Pipeline,
  config: RenderPipelineConfig,
  vertex_shader: Shader<Vertex>,
  fragment_shader: Shader<Fragment>,
}

impl Drop for RenderPipeline {
  fn drop(&mut self) {
    unsafe {
      self.device.destroy_pipeline(self.pipeline, None);
    }
  }
}

impl RenderPipeline {
  pub fn builder() -> RenderPipelineBuilder<VertexShaderMissing, FragmentShaderMissing, ConfigMissing> {
    RenderPipelineBuilder::default()
  }

  fn new(
    vulkan: &Vulkan,
    config: RenderPipelineConfig,
    vertex_shader: Shader<Vertex>,
    fragment_shader: Shader<Fragment>,
  ) -> Result<Self> {
    let pipeline = Self::create_graphics_pipeline(vulkan, vertex_shader.clone(), fragment_shader.clone(), &config)?;

    Ok(Self {
      device: vulkan.logical(),
      pipeline,
      vertex_shader,
      fragment_shader,
      config,
    })
  }

  fn create_graphics_pipeline(
    vulkan: &Vulkan,
    vertex_shader: Shader<Vertex>,
    fragment_shader: Shader<Fragment>,
    config: &RenderPipelineConfig,
  ) -> Result<vk::Pipeline, VulkanError> {
    if config.pipeline_layout == vk::PipelineLayout::null() {
      return Err(unsupported_error!("pipeline layout is null")); // TODO: Add to builder
    }

    if config.render_pass == vk::RenderPass::null() {
      return Err(unsupported_error!("render pass is null")); // TODO: Add to builder
    }

    // TODO: Overhaul pipeline creation to make it more type-driven

    let shader_stage_create_infos = [vertex_shader.pipeline_info(), fragment_shader.pipeline_info()];
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly_info = config.input_assembly_info();
    let viewport_info = config.viewport_info();
    let rasterization_info = config.rasterization_info();
    let multisample_info = config.multisample_info();
    let depth_stencil_info = config.depth_stencil_info();
    let color_blend_info = config.color_blend_info();

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::default()
      .stages(&shader_stage_create_infos)
      .vertex_input_state(&vertex_input_info)
      .input_assembly_state(&input_assembly_info)
      .viewport_state(&viewport_info)
      .rasterization_state(&rasterization_info)
      .multisample_state(&multisample_info)
      .depth_stencil_state(&depth_stencil_info)
      .color_blend_state(&color_blend_info)
      .layout(config.pipeline_layout)
      .render_pass(config.render_pass)
      .subpass(config.subpass);

    unsafe {
      vulkan
        .logical()
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
        .map(|pipelines| pipelines[0])
        .map_err(|err| unsupported_error!("failed to create graphics pipelines: {err:?}"))
    }
  }
}

#[derive(Default, Clone)]
pub struct RenderPipelineConfig {
  pub viewports: Vec<vk::Viewport>,
  pub scissors: Vec<vk::Rect2D>,
  pub color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
  pub pipeline_layout: vk::PipelineLayout,
  pub render_pass: vk::RenderPass,
  pub subpass: u32,
}

impl RenderPipelineConfig {
  pub fn new((width, height): (u32, u32)) -> Self {
    let viewports = vec![vk::Viewport::default()
      .x(0.)
      .y(0.)
      .width(width as f32)
      .height(height as f32)
      .min_depth(0.)
      .max_depth(1.)];

    let scissors = vec![vk::Rect2D::default()
      .offset(vk::Offset2D { x: 0, y: 0 })
      .extent(vk::Extent2D { width, height })];

    let color_blend_attachments = vec![vk::PipelineColorBlendAttachmentState::default()
      .blend_enable(true)
      .src_color_blend_factor(vk::BlendFactor::SRC_COLOR)
      .dst_color_blend_factor(vk::BlendFactor::DST_COLOR)
      .color_blend_op(vk::BlendOp::ADD)
      .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
      .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
      .alpha_blend_op(vk::BlendOp::ADD)
      .color_write_mask(vk::ColorComponentFlags::RGBA)];

    let pipeline_layout = vk::PipelineLayout::null();

    let render_pass = vk::RenderPass::null();

    let subpass: u32 = 0;

    Self {
      viewports,
      scissors,
      color_blend_attachments,
      pipeline_layout,
      render_pass,
      subpass,
    }
  }

  pub fn viewport_info(&self) -> vk::PipelineViewportStateCreateInfo {
    vk::PipelineViewportStateCreateInfo::default()
      .viewports(&self.viewports)
      .scissors(&self.scissors)
  }

  pub fn input_assembly_info(&self) -> vk::PipelineInputAssemblyStateCreateInfo {
    vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_LIST)
  }

  pub fn rasterization_info(&self) -> vk::PipelineRasterizationStateCreateInfo {
    vk::PipelineRasterizationStateCreateInfo::default()
      .cull_mode(vk::CullModeFlags::BACK)
      .line_width(1.0)
  }

  pub fn multisample_info(&self) -> vk::PipelineMultisampleStateCreateInfo {
    vk::PipelineMultisampleStateCreateInfo::default()
      .rasterization_samples(vk::SampleCountFlags::TYPE_1)
      .min_sample_shading(1.0)
  }

  pub fn color_blend_info(&self) -> vk::PipelineColorBlendStateCreateInfo {
    vk::PipelineColorBlendStateCreateInfo::default()
      .logic_op(vk::LogicOp::COPY)
      .attachments(&self.color_blend_attachments)
      .blend_constants([0.0, 0.0, 0.0, 0.0])
  }

  pub fn depth_stencil_info(&self) -> vk::PipelineDepthStencilStateCreateInfo {
    vk::PipelineDepthStencilStateCreateInfo::default()
      .depth_test_enable(true)
      .depth_write_enable(true)
      .depth_compare_op(vk::CompareOp::LESS)
      .max_depth_bounds(1.0)
  }
}

pub struct VertexShaderMissing;
pub struct VertexShaderSpecified(Shader<Vertex>);
pub struct FragmentShaderMissing;
pub struct FragmentShaderSpecified(Shader<Fragment>);

pub struct ConfigMissing;
pub struct ConfigSpecified(RenderPipelineConfig);

pub struct RenderPipelineBuilder<VS, FS, PC> {
  vertex_shader: VS,
  fragment_shader: FS,
  config: PC,
}

impl Default for RenderPipelineBuilder<VertexShaderMissing, FragmentShaderMissing, ConfigMissing> {
  fn default() -> Self {
    Self::new()
  }
}

impl RenderPipelineBuilder<VertexShaderMissing, FragmentShaderMissing, ConfigMissing> {
  pub fn new() -> Self {
    Self {
      vertex_shader: VertexShaderMissing,
      fragment_shader: FragmentShaderMissing,
      config: ConfigMissing,
    }
  }
}

impl<FS, PC> RenderPipelineBuilder<VertexShaderMissing, FS, PC> {
  pub fn with_vertex_shader(self, shader: Shader<Vertex>) -> RenderPipelineBuilder<VertexShaderSpecified, FS, PC> {
    RenderPipelineBuilder {
      vertex_shader: VertexShaderSpecified(shader),
      fragment_shader: self.fragment_shader,
      config: self.config,
    }
  }
}

impl<VS, PC> RenderPipelineBuilder<VS, FragmentShaderMissing, PC> {
  pub fn with_fragment_shader(
    self,
    shader: Shader<Fragment>,
  ) -> RenderPipelineBuilder<VS, FragmentShaderSpecified, PC> {
    RenderPipelineBuilder {
      vertex_shader: self.vertex_shader,
      fragment_shader: FragmentShaderSpecified(shader),
      config: self.config,
    }
  }
}

impl<VS, FS> RenderPipelineBuilder<VS, FS, ConfigMissing> {
  pub fn with_config(self, config: RenderPipelineConfig) -> RenderPipelineBuilder<VS, FS, ConfigSpecified> {
    RenderPipelineBuilder {
      vertex_shader: self.vertex_shader,
      fragment_shader: self.fragment_shader,
      config: ConfigSpecified(config),
    }
  }
}

impl<'c> RenderPipelineBuilder<VertexShaderSpecified, FragmentShaderSpecified, ConfigSpecified> {
  pub fn build(self, vulkan: &Vulkan) -> Result<RenderPipeline> {
    RenderPipeline::new(vulkan, self.config.0, self.vertex_shader.0, self.fragment_shader.0)
      .context("failed to create render pipeline")
  }
}
