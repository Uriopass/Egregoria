#version 450

layout(location=0) in vec3 in_position;
layout(location=1) in vec4 in_color;

layout(location=0) out vec4 out_color;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    out_color = in_color;
    gl_Position = u_view_proj * vec4(in_position, 1.0);
}