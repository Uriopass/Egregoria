#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_depth: texture_multisampled_2d<f32>;
@group(2) @binding(1) var s_depth: sampler;

// wave is (length, amplitude, dir)
fn gerstnerWaveNormal(p: vec2<f32>, t: f32) -> vec3<f32> {
    let N_WAVES: i32 = 6;
    var waves: array<vec4<f32>, 6u> = array<vec4<f32>, 6u>(
    vec4(2.0, 0.05, 0.0, -1.0),
    vec4(4.0, 0.0125, 0.0, 1.0),
    vec4(6.0, 0.0037, 1.0, 1.0),
    vec4(2.0, 0.008, 1.0, 1.0),
    vec4(0.8, 0.001, -0.5, 0.3),
    vec4(0.3, 0.0005, 0.5, -1.0)
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
        @location(2) wpos: vec3<f32>,
        @location(3) _in_uv: vec2<f32>,
        @builtin(position) position: vec4<f32>) -> FragmentOutput {
    let t: f32 = params.time;
    let sun: vec3<f32> = params.sun;
    let cam: vec3<f32> = params.cam_pos.xyz;
//    let normal: vec3<f32> = normalize(vec3<f32>(0.05 * sin(t + wpos.x * 0.01), 0.05 * sin(wpos.y * 0.01), 1.0));
    //let normal: vec3<f32> = vec3(0.0, 0.0, 1.0);
    let normal = normalize(gerstnerWaveNormal(wpos.xy * 0.01, params.time));
    let sun_col: vec3<f32> = params.sun_col.xyz;

    let R: vec3<f32> = normalize(2.0 * normal * dot(normal,sun) - sun);
    let V: vec3<f32> = normalize(cam - wpos);

    let reflect_coeff = 1.0 - dot(V, normal);

    var specular: f32 = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 3.0);

    let reflected: vec3<f32> = reflect(-V, normal);

    let sun_contrib: f32 = clamp(dot(normal, sun), 0.0, 1.0);

    //let base_color: vec3<f32> = vec3<f32>(0.1, 0.25, 0.5 + reflected.z * 0.5);
    let base_color: vec3<f32> = 0.6 * vec3<f32>(0.262, 0.396, 0.508);
    let ambiant: vec3<f32> = 0.05 * base_color;
    //let sunpower: f32 = 0.85 * sun_contrib + 0.5 * specular;
    let sunpower: f32 = 0.95 * reflect_coeff * max(0.0, sqrt(sun.z));

    var final_rgb: vec3<f32> = ambiant + sunpower * (mix(sun_col, base_color, 1.0 - specular));
    //final_rgb = final_rgb + dither(position);

    return FragmentOutput(
        vec4<f32>(final_rgb, 0.9 + reflect_coeff * 0.1),
    );
}