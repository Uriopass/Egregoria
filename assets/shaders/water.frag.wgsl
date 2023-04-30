#include "render_params.wgsl"
#include "atmosphere.wgsl"
#include "tonemap.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_depth: texture_multisampled_2d<f32>;
@group(2) @binding(1) var s_depth: sampler;

@group(3) @binding(0) var t_wavy: texture_2d<f32>;
@group(3) @binding(1) var s_wavy: sampler;

fn sample_depth(coords: vec2<i32>) -> f32 {
    return textureLoad(t_depth, coords, 0).r;
}

// wave is (length, amplitude, dir)
fn gerstnerWaveNormal(p: vec2<f32>, t: f32) -> vec3<f32> {
    let N_WAVES: i32 = 5;
    var waves: array<vec4<f32>, 5u> = array<vec4<f32>, 5u>(
    vec4(2.0, 0.05, 0.0, -1.0),
    vec4(4.0, 0.0125, 0.0, 1.0),
    vec4(6.0, 0.0037, 1.0, 1.0),
    vec4(2.0, 0.008, 1.0, 1.0),
    vec4(0.8, 0.001, -0.5, 0.3)
    );
    let speed: f32 = 0.5;
    let steepness: f32 = 0.3;
	var normal: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);
	for (var i = 0; i < N_WAVES; i++)
	{
	    let ii: i32 = i;
		let wave: vec4<f32> = waves[ii];
		let dir = wave.zw;
		let wi: f32 = 2.0 / wave.x;
		let WA: f32 = wi * wave.y;
		let phi: f32 = speed * wi;
    	let rad: f32 = wi * dot(dir, p) + phi * t;
		let Qi: f32 = steepness / (wave.y * wi * f32(N_WAVES));
		let c_dir: f32 = WA * cos(rad);
		normal.x -= dir.x * c_dir;
		normal.y -= dir.y * c_dir;
		normal.z -= Qi * WA * sin(rad);
	}
	return normal;
}

@fragment
fn frag(@location(0) _in_tint: vec4<f32>,
        @location(1) _in_normal: vec3<f32>,
        @location(2) _in_tangent: vec4<f32>,
        @location(3) wpos: vec3<f32>,
        @location(4) _in_uv: vec2<f32>,
        @builtin(position) position: vec4<f32>) -> FragmentOutput {
    let t: f32 = params.time;
    let sun: vec3<f32> = params.sun;
    let cam: vec3<f32> = params.cam_pos.xyz;
    var normal = gerstnerWaveNormal(wpos.xy * 0.01, params.time_always);
    let sun_col: vec3<f32> = params.sun_col.xyz;

    let wavy: vec3<f32> = textureSample(t_wavy, s_wavy, params.time_always * 0.02 + wpos.xy * 0.001).xyz * 2.0 - 1.0;
    let wavy2: vec3<f32> = textureSample(t_wavy, s_wavy, 30.0 + params.time_always * 0.01 - wpos.yx * vec2(0.001, -0.001)).xyz * 2.0 - 1.0;
    normal = normalize(normal + wavy * 0.15 + wavy2 * 0.1);

    let R: vec3<f32> = normalize(2.0 * normal * dot(normal,sun) - sun);
    let cam_to_wpos: vec3<f32> = cam - wpos;
    let depth: f32 = length(cam_to_wpos);
    let V: vec3<f32> = cam_to_wpos / depth;

    let reflect_coeff = 1.0 - dot(V, normal);

    var specular: f32 = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 5.0);

    let reflected: vec3<f32> = reflect(-V, normal);

    let reflected_atmo = atmosphere(reflected, sun, 1e38);
    let view_atmo = atmosphere(-V, sun, depth * 0.2);

    let sun_contrib: f32 = clamp(dot(normal, sun), 0.0, 1.0);

    let base_color: vec3<f32> = 0.03 * vec3<f32>(0.262, 0.396, 0.508);
    let sunpower: f32 = 0.1 * reflect_coeff;

    var final_rgb: vec3<f32> = tonemap(base_color + view_atmo + sunpower * reflected_atmo);

    return FragmentOutput(
        vec4<f32>(final_rgb, 0.9 + reflect_coeff * 0.1),
    );
}