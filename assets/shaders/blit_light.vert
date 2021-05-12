#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;
layout(location=2) in vec3 in_instance_pos;
layout(location=3) in float in_instance_scale;

layout(location=0) out vec2 out_uv;

layout(set=0, binding=0)
uniform Proj {
    mat4 view_proj;
};

void main() {
    gl_Position = view_proj * vec4(in_pos * in_instance_scale + in_instance_pos, 1.0);

    out_uv = in_uv;
}