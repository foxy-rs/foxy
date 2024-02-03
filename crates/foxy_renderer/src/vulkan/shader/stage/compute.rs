use super::{ShaderDiscriminants, ShaderStage};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComputeShader;

impl ShaderStage for ComputeShader {
  fn kind() -> ShaderDiscriminants {
    ShaderDiscriminants::Compute
  }

  fn default_source() -> String {
    r#""#.into()
  }
}
