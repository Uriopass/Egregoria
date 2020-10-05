#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;

layout(location=0) out vec2 out_uv;

void main() {
    gl_Position = vec4(in_pos.xy, 1.0, 1.0);

    out_uv = in_uv;
}