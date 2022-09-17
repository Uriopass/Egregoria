#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_depth: texture_multisampled_2d<f32>;
@group(2) @binding(1) var s_depth: sampler;

@fragment
fn frag(@location(0) _in_tint: vec4<f32>,
        @location(1) _in_normal: vec3<f32>,
        @location(2) wpos: vec3<f32>,
        @location(3) _in_uv: vec2<f32>,
        @builtin(position) position: vec4<f32>) -> FragmentOutput {
    let t: f32 = params.time;
    let sun: vec3<f32> = params.sun;
    let cam: vec3<f32> = params.cam_pos.xyz;
    let normal: vec3<f32> = normalize(vec3<f32>(0.1 * sin(t + wpos.x * 0.01), 0.1 * sin(wpos.y * 0.01), 1.0));
    let sun_col: vec3<f32> = params.sun_col.xyz;

    let R: vec3<f32> = normalize(2.0 * normal * dot(normal,sun) - sun);
    let V: vec3<f32> = normalize(cam - wpos);

    var specular: f32 = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 2.0);

    let reflected: vec3<f32> = reflect(-V, normal);

    let sun_contrib: f32 = clamp(dot(normal, sun), 0.0, 1.0);

    let base_color: vec3<f32> = vec3<f32>(0.0, 0.0, reflected.z);
    let ambiant: vec3<f32> = 0.15 * base_color;
    //let sunpower: f32 = 0.85 * sun_contrib + 0.5 * specular;
    let sunpower = 0.0;

    var final_rgb: vec3<f32> = ambiant + sunpower * (sun_col * base_color);
    //final_rgb = final_rgb + dither(position);

    return FragmentOutput(
        vec4<f32>(final_rgb, 0.9),
    );
}