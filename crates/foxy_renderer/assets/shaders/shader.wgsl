// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
) -> VertexOutput {
    var VERTICES = array(
      vec4(-0.5, -0.5, 0.0, 1.0),
      vec4( 0.5, -0.5, 0.0, 1.0),
      vec4( 0.0,  0.5, 0.0, 1.0),
    );

    var out: VertexOutput;
    out.clip_position = VERTICES[index];
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}