#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_depth: texture_multisampled_2d<f32>;
@group(2) @binding(1) var s_depth: sampler;



fn hash(x1: vec2<f32> ) -> vec2<f32>  // replace this by something better
{
    let k: vec2<f32> = vec2( 0.3183099, 0.3678794 );
    let x: vec2<f32> = x1*k + k.yx;
    return -1.0 + 2.0*fract( 16.0 * k*fract( x.x*x.y*(x.x+x.y)) );
}


// return gradient noise (in x) and its derivatives (in yz)
fn noised( p: vec2<f32> ) -> vec2<f32>
{
    let i: vec2<f32> = floor( p );
    let f: vec2<f32> = fract( p );

    // quintic interpolation
    let u: vec2<f32> = f*f*f*(f*(f*6.0 - 15.0) + 10.0);
    let du: vec2<f32> = 30.0*f*f*(f*(f - 2.0) + 1.0);

    let ga: vec2<f32> = hash( i + vec2(0.0,0.0) );
    let gb: vec2<f32> = hash( i + vec2(1.0,0.0) );
    let gc: vec2<f32> = hash( i + vec2(0.0,1.0) );
    let gd: vec2<f32> = hash( i + vec2(1.0,1.0) );

    let va: f32 = dot( ga, f - vec2(0.0,0.0) );
    let vb: f32 = dot( gb, f - vec2(1.0,0.0) );
    let vc: f32 = dot( gc, f - vec2(0.0,1.0) );
    let vd: f32 = dot( gd, f - vec2(1.0,1.0) );

    return vec2( //va + u.x*(vb-va) + u.y*(vc-va) + u.x*u.y*(va-vb-vc+vd),   // value
                 ga + u.x*(gb-ga) + u.y*(gc-ga) + u.x*u.y*(ga-gb-gc+gd) +  // derivatives
                 du * (u.yx*(va-vb-vc+vd) + vec2(vb,vc) - va));
}


fn fnoise(pos: vec2<f32>, ampl: f32) -> vec2<f32> {
    let FBM_MAG: f32 = 0.4;
    var dec: vec2<f32> = 70.69 + pos * ampl;

    var amplitude: f32 = 1.0;
    var acc_grad: vec2<f32> = vec2(0.0);

    for (var i = 0; i < 3; i++) {
        let grad: vec2<f32> = noised(dec);
        acc_grad += amplitude * grad;

        dec *= 1.0 / FBM_MAG;
        amplitude *= FBM_MAG;
    }

    return acc_grad;
}

// wave is (length, amplitude, dir)
fn gerstnerWaveNormal(p: vec2<f32>, t: f32) -> vec3<f32> {
    var waves: array<vec4<f32>, 4u> = array<vec4<f32>, 4u>(
    vec4(2.0, 0.1 * 0.5, 0.0, -1.0),
    vec4(4.0, 0.1 * 0.125, 0.0, 1.0),
    vec4(6.0, 0.1 * 0.15*0.25, 1.0, 1.0),
    vec4(2.0, 0.1 * 0.4*0.2, 1.0, 1.0)
    );
    let speed: f32 = 0.5;
    let steepness: f32 = 0.3;
	var normal: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);
    let N_WAVES: i32 = 4;
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
	return normalize(normal);
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
    let normal1 = gerstnerWaveNormal(wpos.xy * 0.01, params.time);
    let off = 0.03 * abs(fnoise(wpos.xy + params.time, 0.01));
    let normal = normalize(vec3(normal1.xy + off.xy, normal1.z));
    //let normal = normalize(vec3(off.xy, 1.0));
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