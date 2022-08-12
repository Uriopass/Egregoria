#include "render_params.wgsl"

struct VertexOutput {
    @location(0) out_pos: vec3<f32>,
    @builtin(position) member: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: RenderParams;

@vertex
fn main(@location(0) in_pos: vec3<f32>, @location(1) in_uv: vec2<f32>) -> VertexOutput {
    let near: vec4<f32> = (params.invproj * vec4(in_pos.xy, -1.0, 1.0));
    let far: vec4<f32> = (params.invproj * vec4(in_pos.xy, 1.0, 1.0));
    let out_pos = near.xyz * far.w - far.xyz * near.w;
    return VertexOutput(out_pos, vec4(in_pos.xy, 0.0, 1.0));
}