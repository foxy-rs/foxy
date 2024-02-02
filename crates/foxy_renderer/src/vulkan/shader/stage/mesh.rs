use super::{ShaderDiscriminants, ShaderStage};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mesh;

impl ShaderStage for Mesh {
  fn kind() -> ShaderDiscriminants {
    ShaderDiscriminants::Mesh
  }

  fn default_source() -> String {
    r#""#.into()
  }
}
