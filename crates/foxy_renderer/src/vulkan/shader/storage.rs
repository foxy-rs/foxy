// pub static SHADERS: OnceLock<ShaderStore> = OnceLock::new();

use std::{collections::HashMap, path::PathBuf};

use foxy_utils::types::handle::Handle;

use super::{stage::ShaderStage, Shader};
use crate::vulkan::device::Device;

#[allow(dead_code)]
pub struct ShaderStore {
  device: Device,
  shaders: Handle<HashMap<PathBuf, Shader>>,
}

impl ShaderStore {
  pub fn delete(&mut self) {
    for shader in self.shaders.get_mut().values_mut() {
      shader.delete(&self.device);
    }
  }
}

impl ShaderStore {
  pub fn new(device: Device) -> Self {
    Self {
      device,
      shaders: Handle::new(Default::default()),
    }
  }

  pub fn get<S: ShaderStage>(&mut self, path: impl Into<PathBuf>) -> Shader {
    let path: PathBuf = path.into();
    let mut shaders = self.shaders.get_mut();
    match shaders.get(&path).cloned() {
      Some(shader) => shader.clone(),
      None => {
        let shader = Shader::new::<S>(self.device.clone(), path.clone());
        shaders.insert(path, shader.clone());
        shader
      }
    }
  }
}
