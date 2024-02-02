// use ash::vk;
// use foxy_utils::types::primitives::Dimensions;

// use super::layout::PipelineLayout;
// use crate::vulkan::error::VulkanError;

// pub struct NoLayout;
// pub struct HasLayout(PipelineLayout);

// pub struct NoRenderPass;
// pub struct HasRenderPass(vk::RenderPass);

// #[derive(Clone)]
// pub struct PipelineConfig<L, R> {
//   pub viewports: Vec<vk::Viewport>,
//   pub scissors: Vec<vk::Rect2D>,
//   pub color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,
//   pub pipeline_layout: L,
//   pub render_pass: R,
//   pub subpass: u32,
// }

// impl PipelineConfig<NoLayout, NoRenderPass> {
//   pub fn new(size: Dimensions) -> Result<Self, VulkanError> {
//     let viewports = vec![vk::Viewport::builder()
//       .x(0.)
//       .y(0.)
//       .width(size.width as f32)
//       .height(size.height as f32)
//       .min_depth(0.)
//       .max_depth(1.)
//       .build()];

//     let scissors = vec![vk::Rect2D::builder()
//       .offset(vk::Offset2D { x: 0, y: 0 })
//       .extent(vk::Extent2D {
//         width: size.width as u32,
//         height: size.height as u32,
//       })
//       .build()];

//     let color_blend_attachments = vec![vk::PipelineColorBlendAttachmentState::builder()
//       .blend_enable(true)
//       .src_color_blend_factor(vk::BlendFactor::SRC_COLOR)
//       .dst_color_blend_factor(vk::BlendFactor::DST_COLOR)
//       .color_blend_op(vk::BlendOp::ADD)
//       .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
//       .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
//       .alpha_blend_op(vk::BlendOp::ADD)
//       .color_write_mask(vk::ColorComponentFlags::RGBA)
//       .build()];

//     // let pipeline_layout = PipelineLayout::;

//     let subpass: u32 = 0;

//     Ok(Self {
//       viewports,
//       scissors,
//       color_blend_attachments,
//       pipeline_layout: NoLayout,
//       render_pass: NoRenderPass,
//       subpass,
//     })
//   }
// }

// impl<R> PipelineConfig<NoLayout, R> {
//   pub fn with_layout(self, pipeline_layout: PipelineLayout) -> PipelineConfig<HasLayout, R> {
//     PipelineConfig {
//       color_blend_attachments: self.color_blend_attachments,
//       viewports: self.viewports,
//       scissors: self.scissors,
//       pipeline_layout: HasLayout(pipeline_layout),
//       render_pass: self.render_pass,
//       subpass: self.subpass,
//     }
//   }
// }

// impl<L> PipelineConfig<L, NoRenderPass> {
//   pub fn with_render_pass(self, render_pass: vk::RenderPass) -> PipelineConfig<L, HasRenderPass> {
//     PipelineConfig {
//       color_blend_attachments: self.color_blend_attachments,
//       viewports: self.viewports,
//       scissors: self.scissors,
//       pipeline_layout: self.pipeline_layout,
//       render_pass: HasRenderPass(render_pass),
//       subpass: self.subpass,
//     }
//   }
// }

// impl PipelineConfig<HasLayout, HasRenderPass> {
//   pub fn layout(&self) -> &PipelineLayout {
//     &self.pipeline_layout.0
//   }

//   pub fn render_pass(&self) -> vk::RenderPass {
//     self.render_pass.0
//   }

//   pub fn viewport_info(&self) -> vk::PipelineViewportStateCreateInfo {
//     vk::PipelineViewportStateCreateInfo::builder()
//       .viewports(&self.viewports)
//       .scissors(&self.scissors)
//       .build()
//   }

//   pub fn input_assembly_info(&self) -> vk::PipelineInputAssemblyStateCreateInfo {
//     vk::PipelineInputAssemblyStateCreateInfo::builder()
//       .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
//       .build()
//   }

//   pub fn rasterization_info(&self) -> vk::PipelineRasterizationStateCreateInfo {
//     vk::PipelineRasterizationStateCreateInfo::builder()
//       .cull_mode(vk::CullModeFlags::BACK)
//       .line_width(1.0)
//       .build()
//   }

//   pub fn multisample_info(&self) -> vk::PipelineMultisampleStateCreateInfo {
//     vk::PipelineMultisampleStateCreateInfo::builder()
//       .rasterization_samples(vk::SampleCountFlags::TYPE_1)
//       .min_sample_shading(1.0)
//       .build()
//   }

//   pub fn color_blend_info(&self) -> vk::PipelineColorBlendStateCreateInfo {
//     vk::PipelineColorBlendStateCreateInfo::builder()
//       .logic_op(vk::LogicOp::COPY)
//       .attachments(&self.color_blend_attachments)
//       .blend_constants([0.0, 0.0, 0.0, 0.0])
//       .build()
//   }

//   pub fn depth_stencil_info(&self) -> vk::PipelineDepthStencilStateCreateInfo {
//     vk::PipelineDepthStencilStateCreateInfo::builder()
//       .depth_test_enable(true)
//       .depth_write_enable(true)
//       .depth_compare_op(vk::CompareOp::LESS)
//       .max_depth_bounds(1.0)
//       .build()
//   }
// }
