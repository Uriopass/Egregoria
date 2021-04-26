#version 450

#include "render_params.glsl"

layout(location=0) in vec3 in_pos;
layout(location=1) in vec3 in_normal;
layout(location=2) in vec2 in_uv;
layout(location=3) in vec4 in_color;

layout(location=4) in vec3 in_instance_pos;
layout(location=5) in vec3 in_instance_dir;
layout(location=6) in vec4 in_instance_tint;

layout(location=0) out vec4 out_color;
layout(location=1) out vec3 out_normal;
layout(location=2) out vec3 out_wpos;
layout(location=3) out vec2 out_uv;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    vec3 x = in_instance_dir;
    vec3 y = cross(vec3(0, 0, 1), x); // Z up
    vec3 z = cross(x, y);

    vec3 off = in_pos.x * x + in_pos.y * y + in_pos.z * z + in_instance_pos;
    vec3 normal = in_normal.x * x + in_normal.y * y + in_normal.z * z;

    gl_Position = u_view_proj * vec4(off, 1.0);

    out_color = in_instance_tint * in_color;
    out_normal = normal;
    out_wpos = off;
    out_uv = in_uv;
}