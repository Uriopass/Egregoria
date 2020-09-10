#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;

layout(location=0) out vec2 out_uv;
layout(location=1) out vec2 out_wv;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 inv_view_proj;
};

void main() {
    gl_Position = vec4(in_pos.xy, 0.0, 1.0);

    out_wv = (inv_view_proj * vec4(in_pos, 1.0)).xy;
    out_uv = in_uv.xy;
}