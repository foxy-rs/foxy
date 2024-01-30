use ash::vk;
use foxy_utils::types::primitives::Dimensions;

use super::layout::PipelineLayout;
use crate::vulkan::error::VulkanError;

pub struct MissingLayout;
pub struct HasLayout(PipelineLayout);

pub struct MissingRenderPass;
pub struct HasRenderPass(vk::RenderPass);

#[derive(Clone)]
pub struct RenderPipelineConfig<L, R> {
  pub viewports: Vec<vk::Viewport>,
  pub scissors: Vec<vk::Rect2D>,
  pub color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
  pub pipeline_layout: L,
  pub render_pass: R,
  pub subpass: u32,
}

impl RenderPipelineConfig<MissingLayout, MissingRenderPass> {
  pub fn new(size: Dimensions) -> Result<Self, VulkanError> {
    let viewports = vec![vk::Viewport::default()
      .x(0.)
      .y(0.)
      .width(size.width as f32)
      .height(size.height as f32)
      .min_depth(0.)
      .max_depth(1.)];

    let scissors = vec![vk::Rect2D::default()
      .offset(vk::Offset2D { x: 0, y: 0 })
      .extent(vk::Extent2D {
        width: size.width as u32,
        height: size.height as u32,
      })];

    let color_blend_attachments = vec![vk::PipelineColorBlendAttachmentState::default()
      .blend_enable(true)
      .src_color_blend_factor(vk::BlendFactor::SRC_COLOR)
      .dst_color_blend_factor(vk::BlendFactor::DST_COLOR)
      .color_blend_op(vk::BlendOp::ADD)
      .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
      .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
      .alpha_blend_op(vk::BlendOp::ADD)
      .color_write_mask(vk::ColorComponentFlags::RGBA)];

    // let pipeline_layout = PipelineLayout::;

    let subpass: u32 = 0;

    Ok(Self {
      viewports,
      scissors,
      color_blend_attachments,
      pipeline_layout: MissingLayout,
      render_pass: MissingRenderPass,
      subpass,
    })
  }
}

impl<R> RenderPipelineConfig<MissingLayout, R> {
  pub fn with_layout(self, pipeline_layout: PipelineLayout) -> RenderPipelineConfig<HasLayout, R> {
    RenderPipelineConfig {
      color_blend_attachments: self.color_blend_attachments,
      viewports: self.viewports,
      scissors: self.scissors,
      pipeline_layout: HasLayout(pipeline_layout),
      render_pass: self.render_pass,
      subpass: self.subpass,
    }
  }
}

impl<L> RenderPipelineConfig<L, MissingRenderPass> {
  pub fn with_render_pass(self, render_pass: vk::RenderPass) -> RenderPipelineConfig<L, HasRenderPass> {
    RenderPipelineConfig {
      color_blend_attachments: self.color_blend_attachments,
      viewports: self.viewports,
      scissors: self.scissors,
      pipeline_layout: self.pipeline_layout,
      render_pass: HasRenderPass(render_pass),
      subpass: self.subpass,
    }
  }
}

impl RenderPipelineConfig<HasLayout, HasRenderPass> {
  pub fn layout(&self) -> &PipelineLayout {
    &self.pipeline_layout.0
  }

  pub fn render_pass(&self) -> vk::RenderPass {
    self.render_pass.0
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
