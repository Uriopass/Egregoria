#include "../render_params.wgsl"
#include "unpack.wgsl"

struct VertexOutput {
    @builtin(position) member: vec4<f32>,
    @location(0) out_normal: vec3<f32>,
    @location(1) out_wpos: vec3<f32>,
#ifdef DEBUG
    @location(2) debug: f32,
#endif
}

struct ChunkData {
    lod: u32,                 // 0 = most details, 1 = half details, 2 = quarter details, etc.
    lod_pow2: u32,            // 2^lod
    resolution: u32,          // number of vertices per side
    distance_lod_cutoff: f32, // max distance at which to switch to the next lod to have smooth transitions
    cell_size: f32,           // size of a cell in world space at lod0
    inv_cell_size: f32,       // 1 / cell_size
}

@group(0) @binding(0) var<uniform> params: RenderParams;

@group(1) @binding(0) var t_terrain: texture_2d<u32>;
@group(1) @binding(1) var s_terrain: sampler;
@group(1) @binding(2) var t_normals: texture_2d<u32>;
@group(1) @binding(3) var s_normals: sampler;
@group(1) @binding(8) var<uniform> cdata: ChunkData;

/*
normal: vec3(self.cell_size * scale as f32, 0.0, hx - height)
                            .cross(vec3(0.0, self.cell_size * scale as f32, hy - height))
                            .normalize(),
*/

fn sampleHeightDxDy(pos: vec2<i32>, lod: i32) -> vec4<f32> {
    let height: f32 = unpack_height(textureLoad(t_terrain, pos, lod).r);
    let normal: vec3<f32> = unpack_normal(textureLoad(t_normals, pos, lod).r);
    return vec4<f32>(height, normal);
}

@vertex
fn vert(@builtin(vertex_index) vid: u32,
        @location(0) in_off: vec2<f32>,
        @location(1) stitch_dir_flags: u32,    // 4 lowest bits are 1 if we need to stitch in that direction. 0 = x+, 1 = y+, 2 = x-, 3 = y-
        ) -> VertexOutput {
    let idx_x: u32 = vid % cdata.resolution;
    let idx_y: u32 = vid / cdata.resolution;

    var in_position: vec2<i32> = vec2(i32(idx_x), i32(idx_y));

    //if (idx_x == 0u) { // x_neg
    //    in_position.y &= -1 << ((stitch_dir_flags & 4u) >> 2u);
    //}
    //else if (idx_x == cdata.resolution - 1u) { // x_pos
    //    in_position.y &= -1 << (stitch_dir_flags & 1u);
    //}
    //if (idx_y == 0u) { // y_neg
    //    in_position.x &= -1 << ((stitch_dir_flags & 8u) >> 3u);
    //}
    //else if (idx_y == cdata.resolution - 1u) { // y_pos
    //    in_position.x &= -1 << ((stitch_dir_flags & 2u) >> 1u);
    //}

    let tpos: vec2<i32> = in_position + vec2<i32>(in_off * cdata.inv_cell_size / f32(cdata.lod_pow2));

    let height_normal: vec4<f32> = sampleHeightDxDy(tpos, i32(cdata.lod));
    var normal: vec3<f32> = height_normal.yzw;

    var world_pos: vec3<f32> = vec3(vec2<f32>(in_position * i32(cdata.lod_pow2)) * cdata.cell_size + in_off, height_normal.x);

#ifdef DEBUG
    var debug = 1.0;
#endif

    let dist_to_cam: f32 = length(params.cam_pos.xyz - vec3(world_pos.xy, 0.0));
    let transition_alpha: f32 = smoothstep(cdata.distance_lod_cutoff * 0.8, cdata.distance_lod_cutoff, dist_to_cam);

    if (cdata.lod < 4u && transition_alpha > 0.0) {
#ifdef DEBUG
//    debug = (f32(cdata.lod) + transition_alpha + 1.0) / 5.0;
    debug = f32(cdata.lod);
#endif
        let height_normal_next: vec4<f32> = sampleHeightDxDy(tpos / 2, i32(cdata.lod) + 1);

        var world_pos_next: vec3<f32> = vec3(vec2<f32>(in_position / 2 * i32(cdata.lod_pow2)) * cdata.cell_size * 2.0 + in_off, height_normal_next.x);


        normal = normalize(mix(normal, height_normal_next.yzw, transition_alpha));
        world_pos = mix(world_pos, world_pos_next, transition_alpha);
    }

    let clip_pos: vec4<f32> = params.proj * vec4(world_pos, 1.0);


    return VertexOutput(clip_pos,
                        normal,
                        world_pos,
                        #ifdef DEBUG
                        debug
                        #endif
                        );
}
