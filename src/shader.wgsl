struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main() -> VertexOutput {
    var out: VertexOutput;
    out.color = vec3<f32>(1.0, 1.0, 1.0);
    out.clip_position = vec4<f32>(1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}