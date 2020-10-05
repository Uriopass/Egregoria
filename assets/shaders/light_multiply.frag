#version 450

layout(location=0) in vec2 in_uv;
layout(location=0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

void main() {
    float light = clamp(texture(sampler2D(t_diffuse, s_diffuse), in_uv).r, 0.0, 1.0);
    vec3 diffuse = vec3(0.1, 0.1, 0.1);
    vec3 yellow =  light * vec3(0.9, 0.9, 0.7);
    out_color = vec4(diffuse + yellow, 1.0);
}