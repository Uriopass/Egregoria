struct Uniforms {
    u_view_proj: mat4x4<f32>,
}

struct VertexOutput {
    @location(0) out_normal: vec3<f32>,
    @location(1) out_wpos: vec3<f32>,
    @builtin(position) member: vec4<f32>,
}

@group(0) @binding(0) var<uniform> global: Uniforms;

@group(2) @binding(0) var t_terraindata: texture_2d<f32>;
@group(2) @binding(1) var s_terraindata: sampler;

/*
normal: vec3(self.cell_size * scale as f32, 0.0, hx - height)
                            .cross(vec3(0.0, self.cell_size * scale as f32, hy - height))
                            .normalize(),
*/

const CELL_SIZE: f32 = 1024.0 / 32.0; // chunk size / chunk resolution

@vertex
fn vert(@location(0) in_position: vec2<f32>, @location(1) in_off: vec2<f32>) -> VertexOutput {
    let tpos: vec2<i32> =  vec2<i32>((in_position + in_off) / CELL_SIZE);
    let height: f32 = textureLoad(t_terraindata, tpos, 0).r;

    let hx: f32 = textureLoad(t_terraindata, vec2<i32>(1, 0) + tpos, 0).r;
    let hy: f32 = textureLoad(t_terraindata, vec2<i32>(0, 1) + tpos, 0).r;

    let pos: vec3<f32> = vec3(in_position + in_off, height);
    let out_normal: vec3<f32> = normalize(cross(vec3(CELL_SIZE, 0.0, hx - height), vec3(0.0, CELL_SIZE, hy - height)));
    let position: vec4<f32> = global.u_view_proj * vec4(pos, 1.0);

    return VertexOutput(out_normal, pos, position);
}
