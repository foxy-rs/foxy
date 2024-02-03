use std::path::PathBuf;

use super::{ShaderDiscriminants, ShaderStage};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GeometryShader;

impl ShaderStage for GeometryShader {
  fn kind() -> ShaderDiscriminants {
    ShaderDiscriminants::Geometry
  }

  fn default_source() -> String {
    r#""#.into()
  }

  fn default_path() -> std::path::PathBuf {
    PathBuf::from("default.geom.glsl")
  }
}
