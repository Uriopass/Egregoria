#version 450

layout(location=0) out vec4 out_color;
layout(location=1) out vec2 out_uv;

struct Sprite {
    vec4 tint;
    vec3 pos;
    vec2 dir;
    vec2 scale;
};

layout (set=2, binding=0) readonly buffer SpriteStorage {
    Sprite sprites[];
};

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

vec2[] ppp = {vec2(0, 0),
              vec2(1, 0),
              vec2(1, 1),

              vec2(0, 0),
              vec2(1, 1),
              vec2(0, 1)};

void main() {
    int aaa = gl_VertexIndex % 6;
    Sprite s = sprites[gl_VertexIndex / 6];

    vec2 in_uv = ppp[aaa];
    vec2 in_pos = (in_uv - vec2(0.5)) * s.scale;

    gl_Position = u_view_proj * vec4(in_pos.x * s.dir - in_pos.y * vec2(s.dir.y, -s.dir.x) + s.pos.xy, s.pos.z, 1.0);

    out_color = s.tint;
    out_uv = in_uv;
}