#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;

layout(location=2) in vec3 in_ipos;
layout(location=3) in vec2 in_irot;
layout(location=4) in vec2 in_iscale;

layout(location=5) in vec4 in_tint;

layout(location=0) out vec2 out_uv;
layout(location=1) out float out_l;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    vec3 scaled = vec3(in_pos.xy * in_iscale, in_pos.z);
    vec3 rotated = vec3(scaled.x * in_irot - scaled.y * vec2(in_irot.y, -in_irot.x), scaled.z);
    gl_Position = u_view_proj * vec4(rotated + in_ipos, 1.0);
    out_uv = in_uv;
    out_l = length((in_iscale.x * in_irot + in_ipos.xy) - in_ipos.xy);
}