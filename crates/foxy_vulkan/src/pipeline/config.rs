use std::sync::Arc;

use ash::vk;
use foxy_types::handle::Handle;

use super::layout::PipelineLayout;
use crate::{device::Device, error::VulkanError};

#[derive(Clone)]
pub struct RenderPipelineConfig {
  pub viewports: Vec<vk::Viewport>,
  pub scissors: Vec<vk::Rect2D>,
  pub color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
  pub pipeline_layout: PipelineLayout,
  pub render_pass: vk::RenderPass,
  pub subpass: u32,
}

impl RenderPipelineConfig {
  pub fn new(device: Handle<Device>, (width, height): (u32, u32)) -> Result<Self, VulkanError> {
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

    let pipeline_layout = PipelineLayout::new(device)?;

    let render_pass = vk::RenderPass::null();

    let subpass: u32 = 0;

    Ok(Self {
      viewports,
      scissors,
      color_blend_attachments,
      pipeline_layout,
      render_pass,
      subpass,
    })
  }

  pub fn with_render_pass(mut self, render_pass: vk::RenderPass) -> Self {
    self.render_pass = render_pass;
    self
  }

  pub fn with_layout(mut self, pipeline_layout: PipelineLayout) -> Self {
    self.pipeline_layout = pipeline_layout;
    self
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
