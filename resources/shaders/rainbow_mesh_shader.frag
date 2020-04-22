#version 450

layout(location=0) in vec4 in_color;
layout(location=1) in vec3 in_pos;

layout(location=0) out vec4 out_color;

layout(set=0, binding=0)
uniform Uniforms {
    float time;
};

void main() {
    float hmm = in_pos.x + in_pos.y;
    vec3 ok = (sin(time * 10.0 + vec3(0.0, 0.5 + in_color.r, 1.0) + in_pos.x + in_pos.y) + 1.0) / 2.0;
    out_color = vec4(ok, 1.0);
}