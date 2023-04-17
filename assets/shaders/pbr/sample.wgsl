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

fn HemisphereSampleuniform(Xi: vec2<f32>) -> vec3<f32> {
    let phi = Xi.x * 2.0 * PI;
    let cosTheta = 1.0 - Xi.y;
    let sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    return vec3<f32>(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);
}

fn HemisphereSampleCos(Xi: vec2<f32>) -> vec3<f32> {
   let phi = Xi.x * 2.0 * PI;
   let cosTheta = sqrt(1.0 - Xi.y);
   let sinTheta = sqrt(1.0 - cosTheta * cosTheta);
   return vec3<f32>(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);
}

fn HemisphereSampleCosRoughness(Xi: vec2<f32>, roughness: f32) -> vec3<f32> {
    let a: f32 = roughness*roughness;

    let phi: f32 = 2.0 * PI * Xi.x;
    let cosTheta: f32 = sqrt((1.0 - Xi.y) / (1.0 + (a*a - 1.0) * Xi.y));
    let sinTheta: f32 = sqrt(1.0 - cosTheta*cosTheta);

    return vec3<f32>(cos(phi) * sinTheta, sin(phi) * sinTheta, cosTheta);
}
