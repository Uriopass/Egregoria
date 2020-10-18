#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;

layout(location=0) out vec2 out_uv;
layout(location=1) out vec2 out_wv;
layout(location=2) out vec3 out_sun;

layout(set = 3, binding = 0) uniform LightParams {
    vec3 ambiant;
    float time;
    mat4 invproj;
};

void main() {
    gl_Position = vec4(in_pos.xy, 1.0, 1.0);

    out_uv = in_uv;
    out_wv = (invproj * vec4(in_pos, 1.0)).xy - vec2(-2000.0, 2000.0);

    float t = 2.0 * 3.1415 * (time - 800.0) / 2400.0;

    vec3 sun = vec3(cos(t), sin(t) * 0.5, sin(t) + 0.5);
    out_sun = sun / length(sun);
}