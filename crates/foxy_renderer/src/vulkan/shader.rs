use std::{ffi::CStr, hash::Hash, path::PathBuf};

use ash::vk;
use strum::{Display, EnumDiscriminants};
use tracing::*;

use self::{source::Source, stage::ShaderStage};
use super::device::Device;
use crate::vulkan::error::VulkanError;

pub mod source;
pub mod stage;
pub mod storage;

enum BuildAttempt {
  First,
  Second,
  Last,
}

#[derive(Display, Clone, Eq, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, Display))]
#[strum_discriminants(enumflags2::bitflags())]
#[strum_discriminants(repr(u32))]
pub enum Shader {
  Vertex { module: vk::ShaderModule },
  Fragment { module: vk::ShaderModule },
  Geometry { module: vk::ShaderModule },
  Compute { module: vk::ShaderModule },
  Mesh { module: vk::ShaderModule },
}

impl PartialEq for Shader {
  fn eq(&self, other: &Self) -> bool {
    self.kind() == other.kind()
  }
}

impl Hash for Shader {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.kind().hash(state);
  }
}

impl Shader {
  pub const SHADER_ASSET_DIR: &'static str = "assets/shaders";
  pub const SHADER_CACHE_DIR: &'static str = "tmp/shaders";

  pub fn delete(&mut self, device: &Device) {
    debug!("Deleting shader module");
    unsafe {
      device.logical().destroy_shader_module(self.module(), None);
    }
  }
}

impl Shader {
  pub fn new<S: ShaderStage>(device: Device, path: impl Into<PathBuf>) -> Self {
    let source = Source::new::<S, _>(path);
    let module = Self::build_shader_module::<S>(&device, &source, BuildAttempt::First)
      .expect("fallbacks should never fail to compile");

    match S::kind() {
      ShaderDiscriminants::Vertex => Self::Vertex { module },
      ShaderDiscriminants::Fragment => Self::Vertex { module },
      ShaderDiscriminants::Geometry => Self::Vertex { module },
      ShaderDiscriminants::Compute => Self::Vertex { module },
      ShaderDiscriminants::Mesh => Self::Vertex { module },
    }
  }

  pub fn kind(&self) -> ShaderDiscriminants {
    ShaderDiscriminants::from(self)
  }

  pub fn module(&self) -> vk::ShaderModule {
    match self {
      Shader::Vertex { module, .. } => *module,
      Shader::Fragment { module, .. } => *module,
      Shader::Geometry { module, .. } => *module,
      Shader::Compute { module, .. } => *module,
      Shader::Mesh { module, .. } => *module,
    }
  }

  pub fn pipeline_info(&self) -> vk::PipelineShaderStageCreateInfo {
    vk::PipelineShaderStageCreateInfo::builder()
      .stage(self.kind().into())
      .module(self.module())
      .name(self.kind().entry_point())
      .build()
  }

  fn build_shader_module<S: ShaderStage>(
    device: &Device,
    source: &Source,
    attempt: BuildAttempt,
  ) -> Result<vk::ShaderModule, VulkanError> {
    match source {
      Source::SPIRV { path, words } => {
        trace!("[{:?}] Building module... {:?}", S::kind(), path);
        // debug!("Words: {:08X?}", words);
        let shader_module = {
          let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(words);

          match unsafe { device.logical().create_shader_module(&shader_module_create_info, None) } {
            Ok(module) => module,
            Err(err) => match attempt {
              BuildAttempt::First => {
                error!("Shader module creation failure, attempting to recompile ({err})");
                let source = Source::new::<S, _>(path);
                Self::build_shader_module::<S>(device, &source, BuildAttempt::Second)?
              }
              BuildAttempt::Second => {
                let source = Source::read_default::<S>();
                Self::build_shader_module::<S>(device, &source, BuildAttempt::Last)?
              }
              BuildAttempt::Last => Err(VulkanError::Shader(
                "Could not recover from shader module creation failure ({err})".into(),
              ))?,
            },
          }
        };

        debug!("[{:?}] Loaded shader.", &path);
        Ok(shader_module)
      }
    }
  }
}

impl ShaderDiscriminants {
  // fn default_source(&self) -> String {
  //   match self {
  //     ShaderDiscriminants::Vertex => Vertex::default_source(),
  //     ShaderDiscriminants::Fragment => Fragment::default_source(),
  //     ShaderDiscriminants::Geometry => Geometry::default_source(),
  //     ShaderDiscriminants::Compute => Compute::default_source(),
  //     ShaderDiscriminants::Mesh => Mesh::default_source(),
  //   }
  // }

  pub fn entry_point(&self) -> &'static CStr {
    // yes, i know this is very redundant, but it might change in the future and i'm
    // lazy and don't want to type this all again
    match self {
      ShaderDiscriminants::Vertex => c"main",
      ShaderDiscriminants::Fragment => c"main",
      ShaderDiscriminants::Geometry => c"main",
      ShaderDiscriminants::Compute => c"main",
      ShaderDiscriminants::Mesh => c"main",
    }
  }
}

impl From<ShaderDiscriminants> for shaderc::ShaderKind {
  fn from(value: ShaderDiscriminants) -> Self {
    match value {
      ShaderDiscriminants::Vertex => shaderc::ShaderKind::Vertex,
      ShaderDiscriminants::Fragment => shaderc::ShaderKind::Fragment,
      ShaderDiscriminants::Compute => shaderc::ShaderKind::Compute,
      ShaderDiscriminants::Geometry => shaderc::ShaderKind::Geometry,
      ShaderDiscriminants::Mesh => shaderc::ShaderKind::Mesh,
    }
  }
}

impl From<ShaderDiscriminants> for vk::ShaderStageFlags {
  fn from(value: ShaderDiscriminants) -> Self {
    match value {
      ShaderDiscriminants::Vertex => vk::ShaderStageFlags::VERTEX,
      ShaderDiscriminants::Fragment => vk::ShaderStageFlags::FRAGMENT,
      ShaderDiscriminants::Compute => vk::ShaderStageFlags::COMPUTE,
      ShaderDiscriminants::Geometry => vk::ShaderStageFlags::GEOMETRY,
      ShaderDiscriminants::Mesh => vk::ShaderStageFlags::MESH_NV,
    }
  }
}
