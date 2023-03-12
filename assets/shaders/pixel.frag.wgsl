#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_albedo: texture_2d<f32>;
@group(2) @binding(1) var s_albedo: sampler;
@group(2) @binding(2) var<uniform> u_metallic: f32;
@group(2) @binding(3) var<uniform> u_roughness: f32;
@group(2) @binding(4) var t_metallic_roughness: texture_2d<f32>;
@group(2) @binding(5) var s_metallic_rougness: sampler;

@group(3) @binding(0) var t_ssao: texture_2d<f32>;
@group(3) @binding(1) var s_ssao: sampler;
@group(3) @binding(2) var t_bnoise: texture_2d<f32>;
@group(3) @binding(3) var s_bnoise: sampler;
@group(3) @binding(4) var t_sun_smap: texture_depth_2d;
@group(3) @binding(5) var s_sun_smap: sampler_comparison;

#include "shadow.wgsl"
#include "render.wgsl"

@fragment
fn frag(@location(0) in_tint: vec4<f32>,
        @location(1) in_normal: vec3<f32>,
        @location(2) in_wpos: vec3<f32>,
        @location(3) in_uv: vec2<f32>,
        @builtin(position) position: vec4<f32>,
        @builtin(front_facing) front_facing: bool,
        ) -> FragmentOutput {

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
    if (params.shadow_mapping_resolution != 0) {
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
    var normal = normalize(in_normal);
    if (!front_facing) {
        normal = -normal;
    }

    var metallic: f32 = 1.0;
    var roughness: f32 = 1.0;
    if (u_metallic == -1.0) {
        let sampled: vec2<f32> = textureSample(t_metallic_roughness, s_metallic_rougness, in_uv).gb;
        roughness = sampled[0];
        metallic = sampled[1];
    } else {
        metallic = u_metallic;
        roughness = u_roughness;
    }

    let c = in_tint * albedo;
    let final_rgb: vec3<f32> = render(params.sun,
                                      params.cam_pos.xyz,
                                      in_wpos,
                                      position.xy,
                                      normal,
                                      c.rgb,
                                      metallic,
                                      roughness,
                                      params.sun_col.rgb,
                                      shadow_v,
                                      ssao);

    return FragmentOutput(vec4<f32>(final_rgb, c.a));
}
