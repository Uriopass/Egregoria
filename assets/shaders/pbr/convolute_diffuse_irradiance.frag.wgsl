struct FragmentOutput {
    @location(0) out_cube: vec4<f32>,
}

struct Params {
    time97: u32, // tick modulo 100
    sample_count: u32,
}

@group(0) @binding(0) var t_environment: texture_cube<f32>;
@group(0) @binding(1) var s_environment: sampler;

@group(1) @binding(0) var<uniform> params: Params;

#include "sample.wgsl"

@fragment
fn frag(@location(0) wpos: vec3<f32>) -> FragmentOutput {
    let normal: vec3<f32> = normalize(wpos);
    let right: vec3<f32> = normalize(cross(vec3(0.0, 0.0, 1.0), normal));
    let up:    vec3<f32> = normalize(cross(normal, right));

    var irradiance: vec3<f32> = vec3(0.0);
    var totalWeight: f32 = 0.0;

    for(var i: u32 = params.time97; i < params.sample_count*97u; i += 97u) {
        let Xi: vec2<f32> = Hammersley(i, params.sample_count*97u);
        let ts: vec3<f32> = HemisphereSampleuniform(Xi); // tangeant space

        let ws: vec3<f32> = ts.x * right + ts.y * up + ts.z * normal; // world space
        irradiance += min(vec3(10.0), textureSampleLevel(t_environment, s_environment, ws, 0.0).rgb);
    }
    irradiance = irradiance / f32(params.sample_count);

    return FragmentOutput(vec4(irradiance.r, irradiance.g, irradiance.b, 0.02));
}