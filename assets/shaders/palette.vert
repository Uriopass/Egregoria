#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec3 in_normal;
layout(location=2) in vec2 in_uv;
layout(location=3) in vec3 in_instance_pos;
layout(location=4) in vec3 in_instance_dir;
layout(location=5) in vec4 in_instance_tint;

layout(location=0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D t_palette;
layout(set = 0, binding = 1) uniform sampler s_palette;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

layout(set = 2, binding = 0) uniform Uni {LightParams params;};

void main() {
    vec3 x = in_instance_dir;
    vec3 y = cross(vec3(0, 0, 1), x); // Z up
    vec3 z = cross(x, y);

    vec3 off = in_pos.x * x + in_pos.y * y + in_pos.z * z;
    vec3 normal = in_normal.x * x + in_normal.y * y + in_normal.z * z;

    gl_Position = u_view_proj * vec4(off + in_instance_pos, 1.0);
    float lol = dot(normal, params.sun);

    vec4 col = texture(sampler2D(t_palette, s_palette), in_uv);
    col.rgb *= 0.2 + 0.8 * lol;
    out_color = col * in_instance_tint;
}