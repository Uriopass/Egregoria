struct FragmentOutput {
    @location(0) out_cube: vec4<f32>,
}

@group(0) @binding(0) var t_environment: texture_cube<f32>;
@group(0) @binding(1) var s_environment: sampler;

@group(1) @binding(0) var<uniform> roughness: f32;

const PI: f32 = 3.141592653589793238462;

fn RadicalInverse_VdC(bitsI: u32) -> f32 {
    var bits = bitsI;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

fn Hammersley(i: u32, N: u32) -> vec2<f32> {
    return vec2(f32(i)/f32(N), RadicalInverse_VdC(i));
}
const SAMPLE_COUNT: u32 = 4096u;

fn ImportanceSampleGGX(Xi: vec2<f32>, N: vec3<f32>, roughness: f32) -> vec3<f32> {
    let a: f32 = roughness*roughness;

    let phi: f32 = 2.0 * PI * Xi.x;
    let cosTheta: f32 = sqrt((1.0 - Xi.y) / (1.0 + (a*a - 1.0) * Xi.y));
    let sinTheta: f32 = sqrt(1.0 - cosTheta*cosTheta);

    // from spherical coordinates to cartesian coordinates
    let H: vec3<f32> = vec3(
       cos(phi) * sinTheta,
       sin(phi) * sinTheta,
       cosTheta
    );

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

    var totalWeight: f32 = 0.0;
    var color: vec3<f32> = vec3(0.0);

    for(var i: u32 = 0u; i < SAMPLE_COUNT; i += 1u) {
        let Xi: vec2<f32> = Hammersley(i, SAMPLE_COUNT);

        let H: vec3<f32>  = ImportanceSampleGGX(Xi, normal, roughness);
        let L: vec3<f32>  = normalize(2.0 * dot(V, H) * H - V);

        let NdotL: f32 = max(dot(normal, L), 0.0);
        if(NdotL > 0.0) {
            color += min(vec3(10.0), textureSampleLevel(t_environment, s_environment, L, 0.0).rgb) * NdotL;
            totalWeight      += NdotL;
        }
    }

    color = color / totalWeight;

    return FragmentOutput(vec4(color.r, color.g, color.b, 1.0));
}