use super::{ShaderKind, StageInfo};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vertex;

impl StageInfo for Vertex {
  fn kind() -> ShaderKind {
    ShaderKind::Vertex
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
}
