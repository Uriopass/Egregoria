#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec4 in_color;
layout(location=2) in vec2 in_uv;

layout(location=0) out vec4 out_color;
layout(location=1) out vec3 out_pos;
layout(location=2) out vec2 out_uv;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    gl_Position = u_view_proj * vec4(in_pos, 1.0);
    out_color = in_color;
    out_pos = in_pos;
    out_uv = in_uv;
}