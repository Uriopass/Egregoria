#version 450
#include "render_params.glsl"

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;

layout(location=0) out vec3 out_pos;

layout(set = 0, binding = 0) uniform Uni {RenderParams params;};

void main() {
    gl_Position = vec4(in_pos.xy, 0.9999999, 1.0);
    vec4 near = (params.invproj * vec4(in_pos.xy, -1.0, 1.0));
    vec4 far = (params.invproj * vec4(in_pos.xy, 1.0, 1.0));
    out_pos = far.xyz / far.w - near.xyz / near.w;
}