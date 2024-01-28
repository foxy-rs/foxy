// pub static SHADERS: OnceLock<ShaderStore> = OnceLock::new();

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use super::{
  stage::{compute::Compute, fragment::Fragment, geometry::Geometry, mesh::Mesh, vertex::Vertex, StageInfo},
  Shader,
};

#[allow(dead_code)]
pub struct ShaderStore {
  device: Arc<ash::Device>,
  vertex_shaders: HashMap<PathBuf, Shader<Vertex>>,
  fragment_shaders: HashMap<PathBuf, Shader<Fragment>>,
  compute_shaders: HashMap<PathBuf, Shader<Compute>>,
  geometry_shaders: HashMap<PathBuf, Shader<Geometry>>,
  mesh_shaders: HashMap<PathBuf, Shader<Mesh>>,
}

impl ShaderStore {
  pub const SHADER_ASSET_DIR: &'static str = "assets/shaders";
  pub const SHADER_CACHE_DIR: &'static str = "tmp/shaders";

  pub fn new(device: Arc<ash::Device>) -> Self {
    Self {
      device,
      vertex_shaders: Default::default(),
      fragment_shaders: Default::default(),
      compute_shaders: Default::default(),
      geometry_shaders: Default::default(),
      mesh_shaders: Default::default(),
    }
  }

  pub fn get_vertex<P: Into<PathBuf>>(&self, path: P) -> Shader<Vertex> {
    self.get_shader(&self.vertex_shaders, path)
  }

  pub fn get_fragment<P: Into<PathBuf>>(&self, path: P) -> Shader<Fragment> {
    self.get_shader(&self.fragment_shaders, path)
  }

  pub fn get_compute<P: Into<PathBuf>>(&self, path: P) -> Shader<Compute> {
    self.get_shader(&self.compute_shaders, path)
  }

  pub fn get_geometry<P: Into<PathBuf>>(&self, path: P) -> Shader<Geometry> {
    self.get_shader(&self.geometry_shaders, path)
  }

  pub fn get_mesh<P: Into<PathBuf>>(&self, path: P) -> Shader<Mesh> {
    self.get_shader(&self.mesh_shaders, path)
  }

  fn get_shader<Stage: StageInfo + Clone, P: Into<PathBuf>>(
    &self,
    shader_map: &HashMap<PathBuf, Shader<Stage>>,
    path: P,
  ) -> Shader<Stage> {
    let path: PathBuf = path.into();
    match shader_map.get(&path) {
      Some(shader) => shader.clone(),
      None => Shader::new(self.device.clone(), path),
    }
  }
}
