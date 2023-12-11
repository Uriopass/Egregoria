#include "../render_params.wgsl"
#include "unpack.wgsl"

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

struct ChunkData {
    lod: u32,                 // 0 = most details, 1 = half details, 2 = quarter details, etc.
    lod_pow2: u32,            // 2^lod
    resolution: u32,          // number of vertices per side
    distance_lod_cutoff: f32, // max distance at which to switch to the next lod to have smooth transitions
    cell_size: f32,           // size of a cell in world space at lod0
    inv_cell_size: f32,       // 1 / cell_size
}

@group(0) @binding(0) var<uniform> global: Uniforms;

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_terraindata: texture_2d<u32>;
@group(2) @binding(1) var s_terraindata: sampler;
@group(2) @binding(4) var<uniform> cdata: ChunkData;

/*
normal: vec3(self.cell_size * scale as f32, 0.0, hx - height)
                            .cross(vec3(0.0, self.cell_size * scale as f32, hy - height))
                            .normalize(),
*/

@vertex
fn vert(@builtin(vertex_index) vid: u32,
        @location(0) in_off: vec2<f32>,
        @location(1) stitch_dir_flags: u32,    // 4 lowest bits are 1 if we need to stitch in that direction. 0 = x+, 1 = y+, 2 = x-, 3 = y-
        ) -> VertexOutput {
    let idx_x: u32 = vid % cdata.resolution;
    let idx_y: u32 = vid / cdata.resolution;

    var in_position: vec2<i32> = vec2(i32(idx_x), i32(idx_y));

    if (idx_x == 0u) { // x_neg
        in_position.y &= -1 << ((stitch_dir_flags & 4u) >> 2u);
    }
    else if (idx_x == cdata.resolution - 1u) { // x_pos
        in_position.y &= -1 << (stitch_dir_flags & 1u);
    }
    if (idx_y == 0u) { // y_neg
        in_position.x &= -1 << ((stitch_dir_flags & 8u) >> 3u);
    }
    else if (idx_y == cdata.resolution - 1u) { // y_pos
        in_position.x &= -1 << ((stitch_dir_flags & 2u) >> 1u);
    }

    let tpos: vec2<i32> = in_position * i32(cdata.lod_pow2) + vec2<i32>(in_off * cdata.inv_cell_size);

    let h_dx_dy: vec3<f32> = unpack(textureLoad(t_terraindata, tpos, 0).r, 1.0);

    let world_pos: vec3<f32> = vec3(vec2<f32>(in_position * i32(cdata.lod_pow2)) * cdata.cell_size + in_off, h_dx_dy.x);
    let clip_pos: vec4<f32> = global.u_view_proj * vec4(world_pos, 1.0);

    //let dist_to_cam: f32 = length(params.cam_pos.xyz - vec3(pos.xy, 0.0));
    //let transition_alpha: f32 = smoothstep(cdata.distance_lod_cutoff * 0.8, cdata.distance_lod_cutoff, dist_to_cam);

    var out_normal: vec3<f32> = normalize(vec3(h_dx_dy.yz, cdata.cell_size * 2.0)); // https://stackoverflow.com/questions/49640250/calculate-normals-from-heightmap

#ifdef DEBUG
    var debug = 0.0;
    debug = f32(cdata.lod);

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
                        clip_pos);
}
