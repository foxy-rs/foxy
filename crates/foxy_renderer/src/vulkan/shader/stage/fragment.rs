use super::{ShaderDiscriminants, ShaderStage};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FragmentShader;

impl ShaderStage for FragmentShader {
  fn kind() -> ShaderDiscriminants {
    ShaderDiscriminants::Fragment
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
