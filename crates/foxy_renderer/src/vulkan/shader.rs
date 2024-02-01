use std::{ffi::CString, marker::PhantomData, path::PathBuf};

use ash::vk;
use tracing::*;

use self::{
  source::Source,
  stage::{ShaderKind, StageInfo},
};
use super::device::Device;
use crate::vulkan::error::VulkanError;

pub mod set;
pub mod source;
pub mod stage;
pub mod storage;

enum BuildAttempt {
  First,
  Second,
  Last,
}

// encapsulate to prevent premature droppage
#[derive(Clone)]
struct Module {
  device: Device,
  module: vk::ShaderModule,
}

impl Module {
  pub fn delete(&mut self) {
    debug!("Deleting shader module");
    unsafe {
      self.device.logical().destroy_shader_module(self.module, None);
    }
  }
}

#[derive(Clone)] // This type is safe to clone because everything is super cheap
pub struct Shader<Stage: StageInfo> {
  shader_entry_point: CString,
  module: Module,
  _p: PhantomData<Stage>,
}

impl<Stage: StageInfo> Shader<Stage> {
  pub fn delete(&mut self) {
    debug!("Deleting shader");
    self.module.delete();
  }
}

impl<Stage: StageInfo> Shader<Stage> {
  pub fn new<P: Into<PathBuf>>(device: Device, path: P) -> Self {
    let source = Source::new::<Stage, _>(path);
    let shader_entry_point = Stage::kind().entry_point_cstring();
    let module =
      Self::build_shader_module(&device, &source, BuildAttempt::First).expect("fallbacks should never fail to compile");

    Self {
      shader_entry_point,
      module: Module { device, module },
      _p: PhantomData,
    }
  }

  pub fn kind(&self) -> ShaderKind {
    Stage::kind()
  }

  pub fn module(&self) -> &vk::ShaderModule {
    &self.module.module
  }

  pub fn pipeline_info(&self) -> vk::PipelineShaderStageCreateInfo {
    vk::PipelineShaderStageCreateInfo::builder()
      .stage(Stage::kind().into())
      .module(self.module.module)
      .name(&self.shader_entry_point)
      .build()
  }

  fn build_shader_module(
    device: &Device,
    source: &Source,
    attempt: BuildAttempt,
  ) -> Result<vk::ShaderModule, VulkanError> {
    match source {
      Source::SPIRV { path, words } => {
        trace!("[{:?}] Building module... {:?}", Stage::kind(), path);
        // debug!("Words: {:08X?}", words);
        let shader_module = {
          let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(words);

          match unsafe { device.logical().create_shader_module(&shader_module_create_info, None) } {
            Ok(module) => module,
            Err(err) => match attempt {
              BuildAttempt::First => {
                error!("Shader module creation failure, attempting to recompile ({err})");
                let source = Source::new::<Stage, _>(path);
                Self::build_shader_module(device, &source, BuildAttempt::Second)?
              }
              BuildAttempt::Second => {
                let source = Source::read_default::<Stage>();
                Self::build_shader_module(device, &source, BuildAttempt::Last)?
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
