use super::{ShaderDiscriminants, ShaderStage};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Compute;

impl ShaderStage for Compute {
  fn kind() -> ShaderDiscriminants {
    ShaderDiscriminants::Compute
  }

  fn default_source() -> String {
    r#""#.into()
  }
}
