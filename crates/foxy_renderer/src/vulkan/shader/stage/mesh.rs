use super::{ShaderKind, StageInfo};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mesh;

impl StageInfo for Mesh {
  fn kind() -> ShaderKind {
    ShaderKind::Mesh
  }

  fn default_source() -> String {
    r#""#.into()
  }
}
