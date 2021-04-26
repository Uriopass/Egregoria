#version 450

#include "render_params.glsl"

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;

layout(location=0) out vec2 out_uv;
layout(location=1) out vec2 out_wv;
layout(location=2) out vec3 out_sun;

layout(set = 1, binding = 0) uniform Uni {RenderParams params;};

void main() {
    gl_Position = vec4(in_pos.xy, 1.0, 1.0);

    out_uv = in_uv;
    out_wv = (params.invproj * vec4(in_pos, 1.0)).xy - vec2(-2000.0, 2000.0);

    float t = 2.0 * 3.1415 * (params.time - 800.0) / 2400.0;
}