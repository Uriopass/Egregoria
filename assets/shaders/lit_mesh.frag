#version 450
#include "light_params.glsl"

layout(location=0) in vec4 in_color;
layout(location=1) in vec3 in_normal;
layout(location=2) in vec3 in_wpos;

layout(location=0) out vec4 out_color;

layout(set = 1, binding = 0) uniform Uni {LightParams params;};

void main() {
    vec3 normal = normalize(in_normal);
    vec3 cam = params.cam_pos.xyz;

    /*
    vec3 L = params.sun.xyz;
    vec3 R = normalize(2 * normal * dot(normal,L) - L);
    vec3 V = normalize(cam - in_wpos);

    float specular = clamp(dot(R, V), 0.0, 1.0);
    specular = pow(specular, 50);
    */
    float diffuse = clamp(dot(normal, params.sun.xyz), 0.0, 1.0);

    vec4 c = in_color;
    c.rgb *= 0.2 + 0.8 * diffuse;
    out_color = c;
}