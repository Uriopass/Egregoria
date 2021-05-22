#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;
layout(location=2) in vec4 in_tint;
layout(location=3) in vec3 in_instance_pos;
layout(location=4) in vec3 in_dir;
layout(location=5) in vec2 in_scale;

layout(location=0) out vec4 out_color;
layout(location=1) out vec3 out_normal;
layout(location=2) out vec3 out_wpos;
layout(location=3) out vec2 out_uv;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    vec3 x = in_dir;
    vec3 y = cross(vec3(0, 0, 1), x); // Z up
    vec3 z = cross(x, normalize(y));

    vec3 scaled = vec3(in_pos.xy * in_scale, in_pos.z);
    vec3 wpos = scaled.x * x + scaled.y * y + scaled.z * z + in_instance_pos;

    gl_Position = u_view_proj * vec4(wpos, 1.0);
    out_color = in_tint;
    out_normal = z;
    out_wpos = wpos;
    out_uv = in_uv;
}