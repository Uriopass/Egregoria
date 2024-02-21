#include "render_params.wgsl"

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
    @location(0) out_ssao: f32,
}

@group(0) @binding(0) var<uniform> params: RenderParams;

#ifdef MSAA
@group(1) @binding(0) var t_depth: texture_multisampled_2d<f32>;
#else
@group(1) @binding(0) var t_depth: texture_2d<f32>;
#endif

@group(1) @binding(1) var s_depth: sampler;


const PHI: f32 = 1.6180340051651;

fn fastnoise(xy: vec2<f32>, seed: f32) -> f32 {
    return fract(tan(distance(xy*PHI, xy)*seed)*xy.x);
}

fn uv2s(uv: vec2<f32>) -> vec2<f32> {
    return round(uv * params.viewport);
}

fn sample_depth(coords: vec2<i32>) -> f32 {
    return textureLoad(t_depth, coords, 0).r;
}

fn derivative(c: vec2<i32>, depth: f32) -> vec2<f32> {
    let depthx: f32 = textureLoad(t_depth, c + vec2<i32>(1, 0), 0).r;
    let depthy: f32 = textureLoad(t_depth, c + vec2<i32>(0, 1), 0).r;

    return vec2(depthx - depth, depthy - depth);
}

var<private> sample_sphere: array<vec3<f32>,16u> = array<vec3<f32>,16u>(
    vec3<f32>( 0.5381, 0.1856,-0.4319), vec3<f32>( 0.1379, 0.2486, 0.4430),
    vec3<f32>( 0.3371, 0.5679,-0.0057), vec3<f32>(-0.6999,-0.0451,-0.0019),
    vec3<f32>( 0.0689,-0.1598,-0.8547), vec3<f32>( 0.0560, 0.0069,-0.1843),
    vec3<f32>(-0.0146, 0.1402, 0.0762), vec3<f32>( 0.0100,-0.1924,-0.0344),
    vec3<f32>(-0.3577,-0.5301,-0.4358), vec3<f32>(-0.3169, 0.1063, 0.0158),
    vec3<f32>( 0.0103,-0.5869, 0.0046), vec3<f32>(-0.0897,-0.4940, 0.3287),
    vec3<f32>( 0.7119,-0.0154,-0.0918), vec3<f32>(-0.0533, 0.0596,-0.5411),
    vec3<f32>( 0.0352,-0.0631, 0.5460), vec3<f32>(-0.4776, 0.2847,-0.0271)
    );

const samples: i32 = 8;
const total_strength: f32 = 0.64;
const radius: f32 = 1.343;
const falloff: f32 = 0.0025;
const base: f32 = 0.01;

@fragment
fn frag(@location(0) in_uv: vec2<f32>) -> FragmentOutput {
    let xr: f32 = fastnoise(in_uv * 1000.0, 1.0);
    let yr: f32 = fastnoise(in_uv * 1000.0, 2.0);
    let zr: f32 = fastnoise(in_uv * 1000.0, 3.0);
    let random: vec3<f32> = normalize( vec3(xr, yr, zr) );

    let pos: vec2<i32> = vec2<i32>(uv2s(in_uv));
    let depth: f32 = sample_depth(pos);

    let derivative: vec2<f32> = derivative(pos, depth);

    let radius_depth: f32 = max(0.001, radius * depth);
    var occlusion: f32 = 0.0;
    for(var i=0; i < samples; i++) {
        let ii: i32 = i;
        let ray: vec3<f32> = radius_depth * reflect(sample_sphere[ii], random);

        let off: vec2<f32> = uv2s(ray.xy);

        let occ_depth: f32 = sample_depth(pos + vec2<i32>(off));
        let difference: f32 = occ_depth - depth;
        var dcorrected: f32 = difference - dot(off, derivative);
        dcorrected = dcorrected / depth;
        //dcorrected = dcorrected - ray.z;

        occlusion += smoothstep(falloff, falloff * 2.0, dcorrected);
    }

    let ao: f32 = 1.0 - total_strength * occlusion * (1.0 / f32(samples));
    let v: f32 = clamp(ao + base, 0.0, 1.0);
    return FragmentOutput(v);
}
