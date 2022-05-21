#version 450

layout(location=0) in vec2 in_position;
layout(location=1) in vec2 in_off;

layout(location=0) out vec3 out_normal;
layout(location=1) out vec3 out_wpos;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

layout(set = 2, binding = 0) uniform texture2D t_terraindata;
layout(set = 2, binding = 1) uniform sampler s_terraindata;

/*
normal: vec3(self.cell_size * scale as f32, 0.0, hx - height)
                            .cross(vec3(0.0, self.cell_size * scale as f32, hy - height))
                            .normalize(),
*/

void main() {
    ivec2 tpos =  ivec2((in_position + in_off) / 32);
    float height = texelFetch(sampler2D(t_terraindata, s_terraindata), tpos, 0).r;

    float hx = texelFetch(sampler2D(t_terraindata, s_terraindata), ivec2(1, 0) + tpos, 0).r;
    float hy = texelFetch(sampler2D(t_terraindata, s_terraindata), ivec2(0, 1) + tpos, 0).r;

    vec3 pos = vec3(in_position + in_off, height);
    out_wpos = pos;
    out_normal = normalize(cross(vec3(32, 0, hx - height), vec3(0, 32, hy - height)));
    gl_Position = u_view_proj * vec4(pos, 1.0);
}