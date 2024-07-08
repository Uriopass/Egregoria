#include "render_params.wgsl"
#include "atmosphere.wgsl"
#include "tonemap.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: RenderParams;

#ifdef MSAA
@group(1) @binding(0) var t_depth: texture_multisampled_2d<f32>;
#else
@group(1) @binding(0) var t_depth: texture_2d<f32>;
#endif
@group(1) @binding(1) var s_depth: sampler;

@group(2) @binding(0) var t_wavy: texture_2d<f32>;
@group(2) @binding(1) var s_wavy: sampler;

@group(3) @binding(0) var t_fog: texture_2d<f32>;
@group(3) @binding(1) var s_fog: sampler;

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
    var normal = gerstnerWaveNormal(wpos.xy * 0.01, params.time_always);

    let wavy: vec3<f32> = textureSample(t_wavy, s_wavy, params.time_always * 0.02 + wpos.xy * 0.001).xyz * 2.0 - 1.0;
    let wavy2: vec3<f32> = textureSample(t_wavy, s_wavy, 30.0 + params.time_always * 0.01 - wpos.yx * vec2(0.001, -0.001)).xyz * 2.0 - 1.0;
    normal = normalize(normal + wavy * 0.15 + wavy2 * 0.1);

    let R: vec3<f32> = normalize(2.0 * normal * dot(normal,params.sun) - params.sun);
    let cam_to_wpos: vec3<f32> = params.cam_pos.xyz - wpos;
    let dist: f32 = length(cam_to_wpos);
    let V: vec3<f32> = cam_to_wpos / dist;

    let reflect_coeff = 1.0 - dot(V, normal);

    var specular: f32 = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 5.0);

    let reflected: vec3<f32> = reflect(-V, normal);

    let reflected_atmo = atmosphere(reflected, params.sun, 1e38);

    let terrain_depth: f32 = 1.0 / textureLoad(t_depth, vec2<i32>(position.xy), 0).x;
    let expected_depth: f32 = -dot(cam_to_wpos, params.cam_dir.xyz);
    let water_depth = (expected_depth - terrain_depth) * params.cam_dir.z;
    let relative_water_depth = clamp(water_depth / 32.0,0.0,1.0);

    let base_factor = mix(0.03, 0.3, pow(1.0-relative_water_depth,4.0));
    let base_color: vec3<f32> = base_factor * vec3<f32>(0.262, 0.396, 0.508);
    let sunpower: f32 = 0.1 * reflect_coeff;

    var final_rgb: vec3<f32> = base_color + sunpower * reflected_atmo;

    #ifdef FOG
    var fog = vec3(0.0);
    var fogdist: vec4<f32> = textureSampleLevel(t_fog, s_fog, position.xy / params.viewport, 0.0);

    if (abs(fogdist.a - dist) > 300.0) {
        #ifdef FOG_DEBUG
        fog = vec3(1.0);
        #else
        fog = atmosphere(-V, params.sun, dist * 0.2);
        #endif
    } else {
        fog = fogdist.rgb;
    }

    final_rgb += fog;
    #endif

    final_rgb = tonemap(final_rgb);

    return FragmentOutput(
        vec4<f32>(final_rgb, 0.9 + reflect_coeff * 0.1),
    );
}