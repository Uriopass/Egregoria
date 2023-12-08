#include "render_params.wgsl"

struct Uniforms {
    u_view_proj: mat4x4<f32>,
}

struct VertexOutput {
    @location(0) out_normal: vec3<f32>,
    @location(1) out_wpos: vec3<f32>,
#ifdef DEBUG
    @location(2) debug: f32,
#endif
    @builtin(position) member: vec4<f32>,
}

struct LevelData {
    lod: u32,
    resolution: u32,
    distance_lod_cutoff: f32, // max distance at which to switch to the next lod to have smooth transitions
    cell_size: f32,
    inv_cell_size: f32,
}

@group(0) @binding(0) var<uniform> global: Uniforms;

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_terraindata: texture_2d<u32>;
@group(2) @binding(1) var s_terraindata: sampler;
@group(2) @binding(4) var<uniform> ldata: LevelData;

/*
normal: vec3(self.cell_size * scale as f32, 0.0, hx - height)
                            .cross(vec3(0.0, self.cell_size * scale as f32, hy - height))
                            .normalize(),
*/

const MAX_HEIGHT: f32 = 1024.0;
const MAX_DIFF: f32 = 32.0;

fn unpack_height(h: u32) -> f32 {
    return ((f32(h) - 32768.0) / 32767.0 ) * MAX_HEIGHT;
}

fn unpack_diffs(v: u32) -> vec2<f32> {
    let x = v & 0xFFu;
    let y = (v & 0xFF00u) >> 8u;
    return vec2<f32>((f32(x) - 128.0) / 127.0 * MAX_DIFF,
                     (f32(y) - 128.0) / 127.0 * MAX_DIFF);
}

@vertex
fn vert(@builtin(vertex_index) vid: u32, @location(0) in_position: vec2<f32>, @location(1) in_off: vec2<f32>) -> VertexOutput {
    let tpos: vec2<i32> =  vec2<i32>((in_position + in_off) * ldata.inv_cell_size);

    let texLoad: vec2<u32> = textureLoad(t_terraindata, tpos, 0).rg;

    let height: f32 = unpack_height(texLoad.r);
    let diffs: vec2<f32> = unpack_diffs(texLoad.g);

    let step: i32 = i32(pow(2.0, f32(ldata.lod)));

    let zf_off: vec2<i32> =  vec2( step * (i32(vid % ldata.resolution) % 2),
                                   step * (i32(vid / ldata.resolution) % 2));

    let world_pos: vec3<f32> = vec3(in_position + in_off, height);

    //let dist_to_cam: f32 = length(params.cam_pos.xyz - vec3(pos.xy, 0.0));
    //let transition_alpha: f32 = smoothstep(ldata.distance_lod_cutoff * 0.8, ldata.distance_lod_cutoff, dist_to_cam);

    var out_normal: vec3<f32> = normalize(vec3(diffs.x, diffs.y, ldata.cell_size * 2.0)); // https://stackoverflow.com/questions/49640250/calculate-normals-from-heightmap

    let position: vec4<f32> = global.u_view_proj * vec4(world_pos, 1.0);

#ifdef DEBUG
    var debug = f32(ldata.lod);

    if(height >= MAX_HEIGHT) {
        debug = diffs.x;
    }
#endif

    return VertexOutput(
                        out_normal,
                        world_pos,
                        #ifdef DEBUG
                        debug,
                        #endif
                        position);
}
