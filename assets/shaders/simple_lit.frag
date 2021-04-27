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

void main() {
    vec4 albedo = texture(sampler2D(t_albedo, s_albedo), in_uv);
    float ssao = 1;
    if (params.ssao) {
       ssao = texture(sampler2D(t_ssao, s_ssao), gl_FragCoord.xy / params.viewport).r;
/*
        if (gl_FragCoord.x > params.viewport.x * 0.5) {
            out_color = vec4(vec3(ssao), 1);
            return;
        }*/
    }
    vec3 normal = normalize(in_normal);
    vec3 cam = params.cam_pos.xyz;

    vec3 L = params.sun.xyz;
    vec3 R = normalize(2 * normal * dot(normal,L) - L);
    vec3 V = normalize(cam - in_wpos);

    float specular = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 5);

    float diffuse = clamp(dot(normal, params.sun.xyz), 0.0, 1.0);

    vec4 c = in_tint * albedo;
    c.rgb *= 0.2 + 0.8 * diffuse + 0.5 * specular;
    c.rgb *= ssao;
    out_color = c;
}