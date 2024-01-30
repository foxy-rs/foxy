use super::{ShaderKind, StageInfo};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Compute;

impl StageInfo for Compute {
  fn kind() -> ShaderKind {
    ShaderKind::Compute
  }

  fn default_source() -> String {
    r#""#.into()
  }
}
