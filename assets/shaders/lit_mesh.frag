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

    float v = dot(normal, params.sun);
    vec4 c = in_color;
    c.rgb *= 0.2 + 0.8 * v;
    out_color = c;
}