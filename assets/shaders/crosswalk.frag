#version 450

layout(location=0) in vec2 in_uv;
layout(location=1) in vec2 in_wv;
layout(location=2) in float in_l;

layout(location=0) out vec4 out_color;

float lerp(float a, float b, float w)
{
    return a + w*(b-a);
}

const float bg = 0.12;
const float fg = 0.8;

void main() {
    float fw = min(0.25, 0.5 * fwidth(in_uv.x) * in_l);
    float invfw = 1.0 / fw;

    float proj = fract(in_uv.x * in_l + fw);
    float gray;

    if (proj < fw) {
        gray = lerp(bg, fg, proj * invfw);
    } else if (proj < 0.5 + fw) {
        gray = fg;
    } else if (proj < 0.5 + fw * 2.0) {
        gray = lerp(fg, bg, (proj - 0.5 - fw) * invfw);
    } else {
        gray = bg;
    }

    if (fw == 0.25) {
        gray = fg * 0.8;
    }

    out_color = 0.8 * vec4(gray, gray, gray, 1.0);
}