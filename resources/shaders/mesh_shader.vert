#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec4 a_color;

layout(location=0) out vec4 v_color;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    v_color = a_color;
    gl_Position = u_view_proj * vec4(a_position, 1.0);
}