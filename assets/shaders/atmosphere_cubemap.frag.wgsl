struct FragmentOutput {
    @location(0) out_cube: vec4<f32>,
}

@group(0) @binding(0) var<uniform> sun_pos: vec4<f32>;

#include "atmosphere.wgsl"

const TAU: f32 = 6.283185307179586476925;

fn cartesian_to_spherical(v: vec3<f32>) -> vec2<f32> {
    let r: f32 = length(v);
    var theta: f32 = acos(v.z / r) / PI;
    var phi: f32 = -atan2(v.y, v.x) / TAU + 0.5;

    return vec2<f32>(phi, theta);
}

@fragment
fn frag(@location(0) wpos: vec3<f32>) -> FragmentOutput {
    let color: vec3<f32> = atmosphere(
        normalize(wpos),                // normalized ray direction
        sun_pos.xyz, // normalized sun direction
        3.40282347E+38,
    );
    let v: vec4<f32> =  vec4(color, 1.0);
    return FragmentOutput(v);
}