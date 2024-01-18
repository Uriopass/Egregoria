#include "render_params.wgsl"

struct VertexOutput {
    @location(0) out_color: vec4<f32>,
    @location(1) out_normal: vec3<f32>,
    @location(2) out_tangent: vec4<f32>,
    @location(3) out_wpos: vec3<f32>,
    @location(4) out_uv: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: RenderParams;

@vertex
fn vert(@location(0) in_pos: vec3<f32>,
        @location(1) in_normal: vec3<f32>,
        @location(2) in_uv: vec2<f32>,
        @location(3) in_color: vec4<f32>,
        @location(4) in_tangent: vec4<f32>,
        @location(5) in_instance_pos: vec3<f32>,
        @location(6) in_instance_dir: vec3<f32>,
        @location(7) in_instance_tint: vec4<f32>) -> VertexOutput {
    let s: f32 = length(in_instance_dir);
    let x: vec3<f32> = in_instance_dir / s;
    let y: vec3<f32> = normalize(vec3(-x.y, x.x, 0.0)); // Z up
    let z: vec3<f32> = normalize(cross(x, y));

    let off: vec3<f32> = s * (in_pos.x * x + in_pos.y * y + in_pos.z * z) + in_instance_pos;
    let normal: vec3<f32> = in_normal.x * x + in_normal.y * y + in_normal.z * z;
    let tangent: vec4<f32> = vec4(in_tangent.x * x + in_tangent.y * y + in_tangent.z * z, in_tangent.w);

    let position: vec4<f32> = global.proj * vec4(off, 1.0);
    let out_color = in_instance_tint * in_color;

    return VertexOutput(out_color, normal, tangent, off, in_uv, position);
}
