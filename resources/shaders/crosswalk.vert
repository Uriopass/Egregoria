#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;
layout(location=2) in mat4 in_model;
layout(location=6) in vec4 in_tint;

layout(location=0) out vec2 out_uv;
layout(location=1) out float out_l;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    gl_Position = u_view_proj * in_model * vec4(in_pos, 1.0);
    out_uv = in_uv;
    out_l = length((in_model * vec4(1.0, 0.0, 0.0, 1.0)).xy - (in_model * vec4(0.0, 0.0, 0.0, 1.0)).xy);
}