struct VertexOutput {
    @location(0) out_uv: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn main(@location(0) in_pos: vec3<f32>,
        @location(1) in_uv: vec2<f32>) -> VertexOutput {
    return VertexOutput(in_uv, vec4(in_pos.xy, 1.0, 1.0));
}
