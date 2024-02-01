use ash::vk;
use foxy_utils::types::handle::Handle;

pub mod config;
pub mod layout;

use self::config::{HasLayout, HasRenderPass, RenderPipelineConfig};
use crate::{
  vulkan::{
    device::Device,
    error::VulkanError,
    shader::set::{HasFragment, HasVertex, NoCompute, NoFragment, NoGeometry, NoMesh, NoVertex, ShaderSet},
  },
  vulkan_error,
  vulkan_unsupported_error,
};

pub trait RenderPipeline {
  #[allow(clippy::type_complexity)]
  type ShaderSet = ShaderSet<Self::Vertex, Self::Fragment, Self::Compute, Self::Geometry, Self::Mesh>;
  type Vertex = NoVertex;
  type Fragment = NoFragment;
  type Compute = NoCompute;
  type Geometry = NoGeometry;
  type Mesh = NoMesh;

  fn new(
    device: Handle<Device>,
    config: RenderPipelineConfig<HasLayout, HasRenderPass>,
    shader_set: Self::ShaderSet,
  ) -> Result<Self, VulkanError>
  where
    Self: Sized;

  fn delete(&mut self);

  fn bind(&self, command_buffer: vk::CommandBuffer);
}

pub struct SimpleRenderPipeline {
  device: Handle<Device>,
  pipeline: vk::Pipeline,
  config: RenderPipelineConfig<HasLayout, HasRenderPass>,
}

impl RenderPipeline for SimpleRenderPipeline {
  type Fragment = HasFragment;
  type Vertex = HasVertex;

  fn new(
    device: Handle<Device>,
    config: RenderPipelineConfig<HasLayout, HasRenderPass>,
    shader_set: Self::ShaderSet,
  ) -> Result<Self, VulkanError>
  where
    Self: Sized,
  {
    let vertex_shader = shader_set.vertex();
    let fragment_shader = shader_set.fragment();

    // TODO: Overhaul pipeline creation to make it more type-driven
    let shader_stage_create_infos = &[vertex_shader.pipeline_info(), fragment_shader.pipeline_info()];
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
    let input_assembly_info = config.input_assembly_info();
    let viewport_info = config.viewport_info();
    let rasterization_info = config.rasterization_info();
    let multisample_info = config.multisample_info();
    let depth_stencil_info = config.depth_stencil_info();
    let color_blend_info = config.color_blend_info();

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
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

    let pipeline = unsafe {
      device
        .get()
        .logical()
        .create_graphics_pipelines(vk::PipelineCache::null(), &[*pipeline_create_info], None)
        .map(|pipelines| pipelines.first().cloned())
        .map_err(|err| vulkan_unsupported_error!("failed to create graphics pipelines: {err:?}"))
    }?
    .ok_or_else(|| vulkan_error!("invalid pipeline index"))?;

    Ok(Self {
      device,
      pipeline,
      config,
    })
  }

  fn delete(&mut self) {
    unsafe {
      self.device.get().logical().destroy_pipeline(self.pipeline, None);
    }
  }

  fn bind(&self, command_buffer: vk::CommandBuffer) {
    unsafe {
      self
        .device
        .get()
        .logical()
        .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline)
    };
  }
}
