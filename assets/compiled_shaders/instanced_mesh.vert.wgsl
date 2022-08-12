struct Uniforms {
    u_view_proj: mat4x4<f32>,
}

struct VertexOutput {
    @location(0) out_color: vec4<f32>,
    @location(1) out_normal: vec3<f32>,
    @location(2) out_wpos: vec3<f32>,
    @location(3) out_uv: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Uniforms;

@vertex 
fn main(@location(0) in_pos: vec3<f32>,
        @location(1) in_normal: vec3<f32>,
        @location(2) in_uv: vec2<f32>,
        @location(3) in_color: vec4<f32>,
        @location(4) in_instance_pos: vec3<f32>,
        @location(5) in_instance_dir: vec3<f32>,
        @location(6) in_instance_tint: vec4<f32>) -> VertexOutput {
    let x: vec3<f32> = in_instance_dir;
    let y: vec3<f32> = cross(vec3(0.0, 0.0, 1.0), x); // Z up
    let z: vec3<f32> = cross(x, normalize(y));

    let off: vec3<f32> = in_pos.x * x + in_pos.y * y + in_pos.z * z + in_instance_pos;
    let normal: vec3<f32> = in_normal.x * x + in_normal.y * y + in_normal.z * z;

    let position: vec4<f32> = global.u_view_proj * vec4(off, 1.0);
    let out_color = in_instance_tint * in_color;

    return VertexOutput(out_color, normal, off, in_uv, position);
}
