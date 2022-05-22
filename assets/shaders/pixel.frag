#version 450
#include "render_params.glsl"

layout(location=0) in vec4 in_tint;
layout(location=1) in vec3 in_normal;
layout(location=2) in vec3 in_wpos;
layout(location=3) in vec2 in_uv;

layout(location=0) out vec4 out_color;

layout(set = 1, binding = 0) uniform Uni {RenderParams params;};

layout(set = 2, binding = 0) uniform texture2D t_albedo;
layout(set = 2, binding = 1) uniform sampler s_albedo;

layout(set = 3, binding = 0) uniform texture2D t_ssao;
layout(set = 3, binding = 1) uniform sampler s_ssao;

layout(set = 3, binding = 2) uniform texture2D t_bnoise;
layout(set = 3, binding = 3) uniform sampler s_bnoise;

layout(set = 3, binding = 4) uniform texture2D t_sun_smap;
layout(set = 3, binding = 5) uniform samplerShadow s_sun_smap;

float dither() {
    float color = texture(sampler2D(t_bnoise, s_bnoise), gl_FragCoord.xy / 512.0).r;
    return (color - 0.5) / 512.0;
}

float sampleShadow() {
    vec4 light_local = params.sunproj * vec4(in_wpos, 1);

    vec3 corrected = light_local.xyz / light_local.w * vec3(0.5, -0.5, 1.0) + vec3(0.5, 0.5, 0.0);

    float total = 0.0;
    float offset = 1.0 / params.shadow_mapping_enabled;

    int x;

    for (int y = -1 ; y <= 1 ; y++) {
        x = -1;
        for (; x <= 1; x++) {
            total += texture(sampler2DShadow(t_sun_smap, s_sun_smap), corrected + offset * vec3(x, y, -1.0));
        }
    }

    total /= 9.0;

    if (light_local.z >= 1.0) {
        return 1.0;
    }
    return mix(total, 1, clamp(dot(light_local.xy, light_local.xy), 0.0, 1.0));
}

void main() {
    vec4 albedo = texture(sampler2D(t_albedo, s_albedo), in_uv);
    float ssao = 1;
    if (params.ssao_enabled != 0) {
       ssao = texture(sampler2D(t_ssao, s_ssao), gl_FragCoord.xy / params.viewport).r;
/*
        if (gl_FragCoord.x > params.viewport.x * 0.5) {
            out_color = vec4(vec3(ssao), 1);
            return;
        }*/
    }

    float shadow_v = 1;
    if (params.shadow_mapping_enabled != 0) {
        shadow_v = sampleShadow();
    }

    /*
    out_color = vec4(in_wpos * 0.001, 1);
    return;
    */
/*
    vec2 p = gl_FragCoord.xy;
    if (p.x < 500 && p.y < 500) {
        out_color = vec4(vec3(texture(sampler2DShadow(t_sun_smap, s_sun_smap), vec3(p / 500, 1))), 1);
        return;
    }*/

    vec3 normal = normalize(in_normal);
    vec3 cam = params.cam_pos.xyz;

    vec3 L = params.sun;
    vec3 R = normalize(2 * normal * dot(normal,L) - L);
    vec3 V = normalize(cam - in_wpos);

    float specular = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 5);

    float sun_contrib = clamp(dot(normal, params.sun), 0.0, 1.0);

    vec4 c = in_tint * albedo;
    vec3 ambiant = 0.15 * c.rgb;
    float sun = (0.85 * sun_contrib + 0.5 * specular) * shadow_v;

    vec3 final_rgb = ambiant;
    final_rgb += sun * (params.sun_col.rgb * c.rgb);
    final_rgb *= ssao;
    final_rgb += dither();
    out_color = vec4(final_rgb, c.a);
}