
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
const SAMPLE_COUNT: u32 = 1024u;
const PI: f32 = 3.141592653589793238462;

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

fn GeometrySchlickGGX(NdotV: f32, roughness: f32) -> f32
{
    let a: f32 = roughness;
    let k: f32 = (a * a) / 2.0;

    let nom: f32   = NdotV;
    let denom: f32 = NdotV * (1.0 - k) + k;

    return nom / denom;
}

fn GeometrySmith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV: f32 = max(dot(N, V), 0.0);
    let NdotL: f32 = max(dot(N, L), 0.0);
    let ggx2: f32 = GeometrySchlickGGX(NdotV, roughness);
    let ggx1: f32 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

fn IntegrateBRDF(NdotV: f32, roughness: f32) -> vec2<f32> {
    let V: vec3<f32> = vec3(
        sqrt(1.0 - NdotV*NdotV),
        0.0,
        NdotV
    );

    var A: f32 = 0.0;
    var B: f32 = 0.0;

    let N: vec3<f32> = vec3(0.0, 0.0, 1.0);

    for(var i: u32 = 0u; i < SAMPLE_COUNT; i += 1u) {
        let Xi: vec2<f32> = Hammersley(i, SAMPLE_COUNT);
        let H: vec3<f32>  = ImportanceSampleGGX(Xi, N, roughness);
        let L: vec3<f32>  = normalize(2.0 * dot(V, H) * H - V);

        let NdotL: f32 = max(L.z, 0.0);
        let NdotH: f32 = max(H.z, 0.0);
        let VdotH: f32 = max(dot(V, H), 0.0);

        if(NdotL > 0.0) {
            let G: f32 = GeometrySmith(N, V, L, roughness);
            let G_Vis: f32 = (G * VdotH) / (NdotH * NdotV);
            let Fc: f32 = pow(1.0 - VdotH, 5.0);

            A += (1.0 - Fc) * G_Vis;
            B += Fc * G_Vis;
        }
    }
    A /= f32(SAMPLE_COUNT);
    B /= f32(SAMPLE_COUNT);
    return vec2<f32>(A, B);
}

struct FragmentOutput {
    @location(0) out_v: vec2<f32>,
}

@fragment
fn frag(@location(0) v_TexCoord: vec2<f32>) -> FragmentOutput {
    return FragmentOutput(IntegrateBRDF(v_TexCoord.x, v_TexCoord.y));
}

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