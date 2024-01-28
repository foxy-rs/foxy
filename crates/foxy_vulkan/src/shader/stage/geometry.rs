use super::{ShaderKind, StageInfo};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Geometry;

impl StageInfo for Geometry {
  fn kind() -> ShaderKind {
    ShaderKind::Geometry
  }

  fn default_source() -> String {
    r#""#.into()
  }
}
