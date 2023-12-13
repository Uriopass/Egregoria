#include "unpack.wgsl"

struct VertexOutput {
    @location(0) v_TexCoord: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@vertex
fn vert(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var tc: vec2<f32> = vec2(0.0, 0.0);
    switch (vi) {
        case 0u: {tc = vec2(1.0, 0.0);}
        case 1u: {tc = vec2(1.0, 1.0);}
        case 2u: {tc = vec2(0.0, 0.0);}
        case 3u: {tc = vec2(0.0, 1.0);}
        default: {}
    }
    let pos: vec2<f32> = tc * 2.0 - 1.0;
    let gl_Position = vec4(pos.x, -pos.y, 0.5, 1.0);

    return VertexOutput(tc, gl_Position);
}


struct FragmentOutput {
    @location(0) o_Target: u32,
}

@group(0) @binding(0) var t_terrain: texture_2d<u32>;

fn pack_diffs(diffs: vec2<f32>, lod_pow2: f32) -> u32 {
    let x = u32((clamp(diffs.x, -MAX_DIFF, MAX_DIFF) / (MAX_DIFF * lod_pow2)) * 127.0 + 128.0);
    let y = u32((clamp(diffs.y, -MAX_DIFF, MAX_DIFF) / (MAX_DIFF * lod_pow2)) * 127.0 + 128.0);
    return (x << 8u) | y;
}

@fragment
fn calc_normals(@location(0) v_TexCoord: vec2<f32>) -> FragmentOutput {
    let dim: vec2<u32> = textureDimensions(t_terrain);

    let id = vec2<u32>(v_TexCoord * vec2<f32>(dim));



    let hR: f32 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(1u, 0u), 0).r);
    let hL: f32 = unpack_height(textureLoad(t_terrain, id - vec2<u32>(1u, 0u), 0).r);
    let hT: f32 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(0u, 1u), 0).r);
    let hB: f32 = unpack_height(textureLoad(t_terrain, id - vec2<u32>(0u, 1u), 0).r);

    let diffs = vec2<f32>((hL - hR), (hB - hT));

    return FragmentOutput(pack_diffs(diffs, 1.0));
}
