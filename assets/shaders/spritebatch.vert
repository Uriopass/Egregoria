#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;
layout(location=2) in vec4 in_tint;
layout(location=3) in vec3 in_instance_pos;
layout(location=4) in vec2 in_dir;
layout(location=5) in vec2 in_scale;

layout(location=0) out vec4 out_color;
layout(location=1) out vec2 out_uv;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    vec2 scaled = in_pos.xy * in_scale;
    gl_Position = u_view_proj * vec4(scaled.x * in_dir - scaled.y * vec2(in_dir.y, -in_dir.x) + in_instance_pos.xy, in_instance_pos.z, 1.0);
    out_color = in_tint;
    out_uv = in_uv;
}