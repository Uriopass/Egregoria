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

fn pack_height(h: f32) -> u32 {
    return u32((h - MIN_HEIGHT) / HEIGHT_RANGE * 65535.0);
}

@group(0) @binding(0) var t_terrain: texture_2d<u32>;

@fragment
fn downsample(@location(0) v_TexCoord: vec2<f32>) -> FragmentOutput {
    let dim = textureDimensions(t_terrain);

    let id = vec2<u32>(v_TexCoord * vec2<f32>(dim));


    let h0 = unpack_height(textureLoad(t_terrain, id, 0).r);
    let h1 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(1u, 0u), 0).r);
    let h2 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(0u, 1u), 0).r);
    let h3 = unpack_height(textureLoad(t_terrain, id - vec2<u32>(1u, 0u), 0).r);
    let h4 = unpack_height(textureLoad(t_terrain, id - vec2<u32>(0u, 1u), 0).r);
    let h5 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(1u, 1u), 0).r);
    let h6 = unpack_height(textureLoad(t_terrain, id - vec2<u32>(1u, 1u), 0).r);
    let h7 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(1u, 0u) - vec2<u32>(0u, 1u), 0).r);
    let h8 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(0u, 1u) - vec2<u32>(1u, 0u), 0).r);

    let gaussian = h0 * 0.25
                 + h1 * 0.125
                 + h2 * 0.125
                 + h3 * 0.125
                 + h4 * 0.125
                 + h5 * 0.0625
                 + h6 * 0.0625
                 + h7 * 0.0625
                 + h8 * 0.0625;
    let maxv = max(max(max(max(max(max(max(max(h0, h1), h2), h3), h4), h5), h6), h7), h8);

    let final_height = (gaussian + maxv) * 0.5;

    return FragmentOutput(pack_height(final_height));
}

@fragment
fn upsample(@builtin(position) pos: vec4<f32>, @location(0) v_TexCoord: vec2<f32>) -> FragmentOutput {
    let p: vec2<f32> = pos.xy - vec2<f32>(0.5);

    let terrain_space = p * 0.5;
    let id = vec2<u32>(terrain_space);

    let h0 = unpack_height(textureLoad(t_terrain, id, 0).r);
    let h1 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(1u, 0u), 0).r);
    let h2 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(0u, 1u), 0).r);
    let h3 = unpack_height(textureLoad(t_terrain, id + vec2<u32>(1u, 1u), 0).r);

    // bilinear interpolation
    let v = fract(terrain_space);

    let h01 = mix(h0, h1, v.x);
    let h23 = mix(h2, h3, v.x);
    let h = mix(h01, h23, v.y);

    let final_height = h;

    return FragmentOutput(pack_height(final_height));
}