#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_albedo: texture_2d<f32>;
@group(2) @binding(1) var s_albedo: sampler;

@group(3) @binding(0) var t_ssao: texture_2d<f32>;
@group(3) @binding(1) var s_ssao: sampler;
@group(3) @binding(2) var t_bnoise: texture_2d<f32>;
@group(3) @binding(3) var s_bnoise: sampler;
@group(3) @binding(4) var t_sun_smap: texture_depth_2d;
@group(3) @binding(5) var s_sun_smap: sampler_comparison;

#include "dither.wgsl"

fn sampleShadow(in_wpos: vec3<f32>) -> f32 {
    let light_local: vec4<f32> = params.sunproj * vec4(in_wpos, 1.0);

    let corrected: vec3<f32> = light_local.xyz / light_local.w * vec3(0.5, -0.5, 1.0) + vec3(0.5, 0.5, 0.0);

    var total: f32 = 0.0;
    let offset: f32 = 1.0 / f32(params.shadow_mapping_enabled);

    var x: i32;

    for (var y = -1 ; y <= 1 ; y++) {
        x = -1;
        for (; x <= 1; x++) {
            let shadow_coord: vec3<f32> = corrected + vec3(f32(x), f32(y), -1.0) * offset;
            total += textureSampleCompare(t_sun_smap, s_sun_smap, shadow_coord.xy, shadow_coord.z);
        }
    }

    total = total / 9.0;

    if (light_local.z >= 1.0) {
        return 1.0;
    }
    return mix(total, 1.0, clamp(dot(light_local.xy, light_local.xy), 0.0, 1.0));
}

@fragment
fn frag(@location(0) in_tint: vec4<f32>,
        @location(1) in_normal: vec3<f32>,
        @location(2) in_wpos: vec3<f32>,
        @location(3) in_uv: vec2<f32>,
        @builtin(position) position: vec4<f32>) -> FragmentOutput {
    let albedo: vec4<f32> = textureSample(t_albedo, s_albedo, in_uv);
    var ssao = 1.0;
    if (params.ssao_enabled != 0) {
       ssao = textureSample(t_ssao, s_ssao, position.xy / params.viewport).r;
/*
        if (position.x > params.viewport.x * 0.5) {
            out_color = vec4(vec3(ssao), 1);
            return;
        }*/
    }

    var shadow_v: f32 = 1.0;
    if (params.shadow_mapping_enabled != 0) {
        shadow_v = sampleShadow(in_wpos);
    }

    /*
    out_color = vec4(in_wpos * 0.001, 1);
    return;
    */
/*
    vec2 p = position.xy;
    if (p.x < 500 && p.y < 500) {
        out_color = vec4(vec3(texture(sampler2DShadow(t_sun_smap, s_sun_smap), vec3(p / 500, 1))), 1);
        return;
    }*/

    let normal: vec3<f32> = normalize(in_normal);
    let cam: vec3<f32> = params.cam_pos.xyz;

    let L: vec3<f32> = params.sun;
    let R: vec3<f32> = normalize(2.0 * normal * dot(normal,L) - L);
    let V: vec3<f32> = normalize(cam - in_wpos);

    var specular: f32 = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 5.0);

    let sun_contrib: f32 = clamp(dot(normal, params.sun), 0.0, 1.0);

    let c: vec4<f32> = in_tint * albedo;
    let ambiant: vec3<f32> = 0.15 * c.rgb;
    let sun: f32 = (0.85 * sun_contrib + 0.5 * specular) * shadow_v;

    var final_rgb: vec3<f32> = ambiant;
    final_rgb = final_rgb + sun * (params.sun_col.rgb * c.rgb);
    final_rgb = final_rgb * ssao;
    final_rgb = final_rgb + dither(position.xy);
    let out_color: vec4<f32> = vec4(final_rgb, c.a);

    return FragmentOutput(out_color);
}
