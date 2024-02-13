struct FragmentOutput {
    @location(0) out_cube: vec4<f32>,
}

struct Params {
    roughness: f32,
    time97: u32, // tick modulo 97
    sample_count: u32,
}

@group(0) @binding(0) var t_environment: texture_cube<f32>;
@group(0) @binding(1) var s_environment: sampler;

@group(1) @binding(0) var<uniform> params: Params;

#include "sample.wgsl"

fn ImportanceSampleGGX(Xi: vec2<f32>, N: vec3<f32>, roughness: f32) -> vec3<f32> {
    let H: vec3<f32> = HemisphereSampleCosRoughness(Xi, roughness);

    // from tangent-space vector to world-space sample vector
    let up: vec3<f32>        = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), abs(N.z) < 0.999);
    let tangent: vec3<f32>   = normalize(cross(up, N));
    let bitangent: vec3<f32> = cross(N, tangent);

    let sampleVec: vec3<f32> = tangent * H.x + bitangent * H.y + N * H.z;
    return normalize(sampleVec);
}

@fragment
fn frag(@location(0) wpos: vec3<f32>) -> FragmentOutput {
    let normal: vec3<f32> = normalize(wpos);
    let R: vec3<f32> = normal;
    let V: vec3<f32> = R;

    if (params.roughness == 0.0) {
        return FragmentOutput(vec4(min(vec3(10.0), textureSampleLevel(t_environment, s_environment, normal, 0.0).rgb), 1.0));
    }

    var totalWeight: f32 = 0.0;
    var color: vec3<f32> = vec3(0.0);

    for(var i: u32 = params.time97; i < params.sample_count*97u; i += 97u) {
        let Xi: vec2<f32> = Hammersley(i, params.sample_count*97u);

        let H: vec3<f32>  = ImportanceSampleGGX(Xi, normal, params.roughness);
        let L: vec3<f32>  = normalize(2.0 * dot(V, H) * H - V);

        let NdotL: f32 = max(dot(normal, L), 0.0);
        if(NdotL > 0.0) {
            color += min(vec3(10.0), textureSampleLevel(t_environment, s_environment, L, 0.0).rgb) * NdotL;
            totalWeight      += NdotL;
        }
    }
    if (totalWeight == 0.0) {
        totalWeight = 1.0;
    }
    color = color / totalWeight;

    return FragmentOutput(vec4(color, 0.04));
}