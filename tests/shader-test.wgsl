// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main() -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(1.0, 2.0, 3.0, 4.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 2.0, 3.0, 4.0);
}