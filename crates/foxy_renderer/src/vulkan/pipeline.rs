use std::collections::HashSet;

use strum::{Display, EnumDiscriminants};
use tracing::debug;

pub mod config;
pub mod layout;

use self::layout::PipelineLayout;
use super::{
  device::Device,
  shader::{Shader, ShaderDiscriminants},
};
use crate::{vulkan::error::VulkanError, vulkan_error};

/// Tag for Pipeline and PipelineLayout enums
pub struct GraphicsPipeline;
impl PipelineType for GraphicsPipeline {
  fn kind() -> PipelineDiscriminants {
    PipelineDiscriminants::Graphics
  }
}

/// Tag for Pipeline and PipelineLayout enums
pub struct ComputePipeline;
impl PipelineType for ComputePipeline {
  fn kind() -> PipelineDiscriminants {
    PipelineDiscriminants::Compute
  }
}

/// Tag for Pipeline and PipelineLayout enums
pub trait PipelineType {
  fn kind() -> PipelineDiscriminants;
}

#[derive(EnumDiscriminants)]
#[strum_discriminants(derive(Hash, Display))]
pub enum Pipeline {
  Graphics { pipeline: vk::Pipeline },
  Compute { pipeline: vk::Pipeline },
}

impl Pipeline {
  pub fn new<P: PipelineType>(
    device: &Device,
    shaders: HashSet<Shader>,
    layout: &PipelineLayout,
  ) -> Result<Self, VulkanError> {
    Ok(match P::kind() {
      PipelineDiscriminants::Graphics => {
        unimplemented!("graphics pipelines aren't implemented")
      }
      PipelineDiscriminants::Compute => {
        let Some(shader) = shaders.iter().find(|&s| {
          debug!("{:?}", s.kind());
          s.kind() == ShaderDiscriminants::Compute
        }) else {
          return Err(VulkanError::MismatchedShaders);
        };

        let shader_info = shader.pipeline_info();
        let pipeline_create_info = vk::ComputePipelineCreateInfo::builder()
          .stage(shader_info)
          .layout(layout.layout());
        let pipeline = unsafe {
          device
            .logical()
            .create_compute_pipelines(vk::PipelineCache::null(), &[*pipeline_create_info], None)
            .map(|pipelines| pipelines.first().cloned())
            .map_err(|e| e.1)
        }?
        .ok_or_else(|| vulkan_error!("invalid pipeline index"))?;

        Self::Compute { pipeline }
      }
    })
  }

  pub fn delete(&mut self, device: &Device) {
    unsafe {
      device.logical().destroy_pipeline(self.pipeline(), None);
    }
  }

  pub fn pipeline(&self) -> vk::Pipeline {
    match self {
      Pipeline::Graphics { pipeline } => *pipeline,
      Pipeline::Compute { pipeline } => *pipeline,
    }
  }

  pub fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer) {
    match self {
      Pipeline::Graphics { pipeline } => {
        unsafe {
          device
            .logical()
            .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, *pipeline)
        };
      }
      Pipeline::Compute { pipeline } => {
        unsafe {
          device
            .logical()
            .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, *pipeline)
        };
      }
    }
  }
}
