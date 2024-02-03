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
}
