#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: RenderParams;

@group(1) @binding(0) var t_bnoise: texture_2d<f32>;
@group(1) @binding(1) var s_bnoise: sampler;

@group(2) @binding(0) var t_starfield: texture_2d<f32>;
@group(2) @binding(1) var s_starfield: sampler;
@group(2) @binding(2) var t_environment: texture_cube<f32>;
@group(2) @binding(3) var s_environment: sampler;

#include "dither.wgsl"
#include "atmosphere.wgsl"
#include "tonemap.wgsl"

@fragment
fn frag(@location(0) in_pos: vec3<f32>, @builtin(position) position: vec4<f32>) -> FragmentOutput {
    var fsun: vec3<f32> = params.sun;
    var pos: vec3<f32> = normalize(in_pos.xyz);

    let longitude: f32 = atan2(pos.x, pos.y);

    var color: vec3<f32> = atmosphere(
        pos,           // normalized ray direction
        fsun,          // normalized sun direction
        3.40282347E+38,
    );

    //color = textureSampleLevel(t_environment, s_environment, pos, 0.0).rgb;

    color = color + max(pos.z + 0.1, 0.0) * 5.0 * textureSample(t_starfield, s_starfield, vec2(longitude, pos.z)).rgb; // starfield
    color = color + max(pos.z, 0.0) * 10000.0 * smoothstep(0.99993, 1.0, dot(fsun, pos)); // sun

    var ocrgb = tonemap(color);
    ocrgb = ocrgb + dither(position.xy);
    return FragmentOutput(vec4(ocrgb.r, ocrgb.g, ocrgb.b, 1.0));
}

struct VertexOutput {
    @location(0) out_pos: vec3<f32>,
    @builtin(position) member: vec4<f32>,
}

@vertex
fn vert(@location(0) in_pos: vec3<f32>, @location(1) in_uv: vec2<f32>) -> VertexOutput {
    let near: vec4<f32> = (params.invproj * vec4(in_pos.xy, -1.0, 1.0));
    let far: vec4<f32> = (params.invproj * vec4(in_pos.xy, 1.0, 1.0));
    let out_pos = near.xyz * far.w - far.xyz * near.w;
    return VertexOutput(out_pos, vec4(in_pos.xy, 0.0, 1.0));
}