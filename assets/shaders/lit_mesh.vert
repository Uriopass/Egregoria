#version 450

layout(location=0) in vec3 in_position;
layout(location=1) in vec3 in_normal;
layout(location=2) in vec4 in_color;

layout(location=0) out vec4 out_color;
layout(location=1) out vec3 out_normal;
layout(location=2) out vec3 out_wpos;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    out_wpos = in_position;
    out_color = in_color;
    out_normal = in_normal;
    gl_Position = u_view_proj * vec4(in_position, 1.0);
}