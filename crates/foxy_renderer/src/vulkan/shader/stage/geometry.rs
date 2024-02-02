use super::{ShaderDiscriminants, ShaderStage};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Geometry;

impl ShaderStage for Geometry {
  fn kind() -> ShaderDiscriminants {
    ShaderDiscriminants::Geometry
  }

  fn default_source() -> String {
    r#""#.into()
  }
}
