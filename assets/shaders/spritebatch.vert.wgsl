#include "render_params.wgsl"

struct VertexOutput {
    @location(0) out_color: vec4<f32>,
    @location(1) out_normal: vec3<f32>,
    @location(2) out_tangent: vec4<f32>,
    @location(3) out_wpos: vec3<f32>,
    @location(4) out_uv: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@group(0) @binding(0) var<uniform> globals: RenderParams;

@vertex
fn vert(@location(0) in_pos: vec3<f32>,
        @location(1) in_uv: vec2<f32>,
        @location(2) in_tint: vec4<f32>,
        @location(3) in_instance_pos: vec3<f32>,
        @location(4) in_dir: vec3<f32>,
        @location(5) in_scale: vec2<f32>) -> VertexOutput {
    let x: vec3<f32> = in_dir;
    let y: vec3<f32> = cross(vec3(0.0, 0.0, 1.0), x); // Z up
    let z: vec3<f32> = cross(x, normalize(y));

    let scaled: vec3<f32> = vec3(in_pos.xy * in_scale, in_pos.z);
    let wpos: vec3<f32> = scaled.x * x + scaled.y * y + scaled.z * z + in_instance_pos;

    let position = globals.proj * vec4(wpos, 1.0);

    return VertexOutput(in_tint, z, vec4(0.0), wpos, in_uv, position);
}
