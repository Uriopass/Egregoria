#version 450

layout(location=0) in vec4 in_color;
layout(location=0) out vec4 out_color;
layout(location=1) out vec4 out_normal;

void main() {
    out_color = in_color;
    out_normal = vec4(0.0);
}