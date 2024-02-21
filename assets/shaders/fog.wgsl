#include "render_params.wgsl"
#include "atmosphere.wgsl"

struct VertexOutput {
    @location(0) out_uv: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@vertex
fn vert(@location(0) in_pos: vec3<f32>,
        @location(1) in_uv: vec2<f32>) -> VertexOutput {
    return VertexOutput(in_uv, vec4(in_pos.xy, 1.0, 1.0));
}

struct FragmentOutput {
    @location(0) out_fog: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: RenderParams;

#ifdef MSAA
@group(1) @binding(0) var t_depth: texture_multisampled_2d<f32>;
#else
@group(1) @binding(0) var t_depth: texture_2d<f32>;
#endif
@group(1) @binding(1) var s_depth: sampler;

fn uv2s(uv: vec2<f32>) -> vec2<f32> {
    return round(uv * params.viewport);
}

fn sample_depth(coords: vec2<i32>) -> f32 {
    return textureLoad(t_depth, coords, 0).r;
}

@fragment
fn frag(@location(0) in_uv: vec2<f32>) -> FragmentOutput {
    let pos: vec2<i32> = vec2<i32>(uv2s(in_uv));
    var depth: f32 = sample_depth(pos);

    if (depth <= 0.00001) {
        depth = 0.00001;
    }

    let uv = vec2(in_uv.x * 2.0 - 1.0, -in_uv.y * 2.0 + 1.0);
    let wposP = params.invproj * vec4<f32>(uv, depth, 1.0);

    let wpos = wposP.xyz / wposP.w;

    let diff = params.cam_pos.xyz - wpos;
    var dist = length(diff);
    let V = diff / dist;

    if (depth <= 0.00001) {
        dist = 1e38;
    } else {
        #ifndef FOG
        discard;
        #endif
    }
    let atmo: vec3<f32> = atmosphere(-V, params.sun, dist * 0.2);

    return FragmentOutput(vec4(atmo, dist));
}
