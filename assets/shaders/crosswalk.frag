#version 450

layout(location=0) in vec2 in_uv;
layout(location=1) in vec2 in_wv;
layout(location=2) in float in_l;

layout(location=0) out vec4 out_color;

const float bg = 0.052;
const float fg = 0.44;

void main() {
    float fw = min(0.45, fwidth(in_uv.x) * in_l);
    float invfw = 1.0 / fw;

    float proj = fract(in_uv.x * in_l + fw);

    float gray = bg;
    gray = (proj < 0.5 + fw) ? mix(fg, bg, (proj - 0.5) * invfw) : gray;
    gray = (proj < 0.5)      ? fg                                : gray;
    gray = (proj < fw)       ? mix(bg, fg, proj * invfw)         : gray;

    gray = (fw  == 0.45)     ? fg * 0.6                          : gray;

    out_color = vec4(gray, gray, gray, 1.0);
}