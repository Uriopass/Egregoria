#version 450

layout(location=0) in vec4 in_color;
layout(location=1) in vec3 in_normal;

layout(location=0) out vec4 out_color;

void main() {
    vec3 normal = normalize(in_normal);

    float v = dot(normal, vec3(0.577, 0.576, 0.577));
    vec4 c = in_color;
    c.rgb *= v;
    out_color = c;
}