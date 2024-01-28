use super::{ShaderKind, StageInfo};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fragment;

impl StageInfo for Fragment {
  fn kind() -> ShaderKind {
    ShaderKind::Fragment
  }

  fn default_source() -> String {
    r#"#version 460

    layout (location = 0) out vec4 out_color;

    void main() {
      out_color = vec4(1.0, 1.0, 1.0, 1.0);
    }
    "#
    .into()
  }
}
