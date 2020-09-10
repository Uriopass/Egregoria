#version 450

layout(location=0) in vec2 in_uv;
layout(location=1) in vec2 in_wv;

layout(location=0) out vec4 out_color;

void main() {
    out_color = vec4(in_wv.xy * 0.01, 0.5, 1.0);
}

