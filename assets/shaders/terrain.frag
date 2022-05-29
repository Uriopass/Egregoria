#version 450
#include "render_params.glsl"

layout(location=0) in vec3 in_normal;
layout(location=1) in vec3 in_wpos;

layout(location=0) out vec4 out_color;

layout(set = 1, binding = 0) uniform Uni {RenderParams params;};

layout(set = 2, binding = 0) uniform texture2D t_terraindata;
layout(set = 2, binding = 1) uniform sampler s_terraindata;

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

float grid() {
    float level = fwidth(in_wpos.x)*20;//length(vec2(dFdx(in_wpos.x), dFdy(in_wpos.x))) * 0.02;

    float w = 10000;
    float isIn = 0.0;
    vec2 curgrid = in_wpos.xy / 10000;

    while(w > level*100) {
        w /= 10;
        curgrid *= 10;
    }

    while(w > level) {
        vec2 moved = fract(curgrid);
        float v = min(min(moved.x, moved.y), min(1 - moved.x, 1 - moved.y));

        float isOk = (1 - smoothstep(0.004, 0.00415, v)) * 2 * (1 - smoothstep(level*100*0.5, level*100, w));
        isIn = max(isIn, isOk);
        w /= 10;
        curgrid *= 10;
    }
    return isIn;
}

void main() {
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

    /*
        let col: LinearColor = if height < -20.0 {
        common::config().sea_col.into()
    } else if height < 0.0 {
        common::config().sand_col.into()
    } else {
        0.37 * LinearColor::from(common::config().grass_col)
    };
        */

    vec4 c = params.grass_col;

    if (params.grid_enabled != 0) {
        c.g += grid() * 0.015;
    }

    c = mix(params.sand_col, c, smoothstep(-5.0, 0.0, in_wpos.z));
    c = mix(params.sea_col, c, smoothstep(-25.0, -20.0, in_wpos.z));

    vec3 normal = normalize(in_normal);
    vec3 cam = params.cam_pos.xyz;

    vec3 L = params.sun;
    vec3 R = normalize(2 * normal * dot(normal,L) - L);
    vec3 V = normalize(cam - in_wpos);

    float specular = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 2);

    float sun_contrib = clamp(dot(normal, params.sun), 0.0, 1.0);

    vec3 ambiant = 0.15 * c.rgb;
    float sun = (0.85 * sun_contrib + 0.5 * specular) * shadow_v;

    vec3 final_rgb = ambiant;
    final_rgb += sun * (params.sun_col.rgb * c.rgb);
    final_rgb *= ssao;
    final_rgb += dither();
    out_color = vec4(final_rgb, c.a);
}