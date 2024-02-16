use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderHandle(pub PathBuf);

impl From<&str> for ShaderHandle {
  fn from(value: &str) -> Self {
    Self(value.into())
  }
}
