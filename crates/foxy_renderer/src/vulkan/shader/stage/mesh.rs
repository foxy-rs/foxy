use std::path::PathBuf;

use super::{ShaderDiscriminants, ShaderStage};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshShader;

impl ShaderStage for MeshShader {
  fn kind() -> ShaderDiscriminants {
    ShaderDiscriminants::Mesh
  }

  fn default_source() -> String {
    r#""#.into()
  }

  fn default_path() -> std::path::PathBuf {
    PathBuf::from("default.mesh.glsl")
  }
}
