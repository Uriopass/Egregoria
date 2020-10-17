#version 450

layout(location=0) in vec3 in_pos;
layout(location=1) in vec2 in_uv;

layout(location=0) out vec2 out_uv;
layout(location=1) out vec2 out_wv;
layout(location=2) out float out_zoom;
layout(location=3) out float out_time;

layout(set=0, binding=0)
uniform Proj {
    mat4 inv_view_proj;
};

layout(set=1, binding=0)
uniform Time {
    float in_time;
};

void main() {
    gl_Position = vec4(in_pos.xy, 0.0, 1.0);

    out_wv = (inv_view_proj * vec4(in_pos, 1.0)).xy - vec2(-2000.0, 2000.0);
    out_uv = in_uv.xy;
    out_zoom = 1.0 / inv_view_proj[0][0];
    out_time = in_time;
}