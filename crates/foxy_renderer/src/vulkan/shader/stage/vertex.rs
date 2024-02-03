use std::path::PathBuf;

use super::{ShaderDiscriminants, ShaderStage};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct VertexShader;

impl ShaderStage for VertexShader {
  fn kind() -> ShaderDiscriminants {
    ShaderDiscriminants::Vertex
  }

  fn default_source() -> String {
    r#"#version 460

    vec2 positions[3] = vec2[](
      vec2(0.0, -0.5),
      vec2(0.5, 0.5),
      vec2(-0.5, 0.5)
    );

    void main() {
      gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
    }
    "#
    .into()
  }

  fn default_path() -> std::path::PathBuf {
    PathBuf::from("default.vert.glsl")
  }
}
