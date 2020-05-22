#version 450

layout(location=0) in vec2 in_uv;
layout(location=1) in float in_l;

layout(location=0) out vec4 out_color;

float lerp(float a, float b, float w)
{
    return a + w*(b-a);
}

void main() {
    float fw = min(0.25, 0.5 * fwidth(in_uv.x) * in_l);
    float invfw = 1.0 / fw;

    float proj = fract(in_uv.x * in_l + fw);
    float gray;

    if (proj < fw) {
        gray = lerp(0.21404114, 1.0, proj * invfw);
    } else if (proj < 0.5 + fw) {
        gray = 1.0;
    } else if (proj < 0.5 + fw * 2.0) {
        gray = lerp(1.0, 0.21404114, (proj - 0.5 - fw) * invfw);
    } else {
        gray = 0.21404114;
    }

    if (fw == 0.25) {
        gray = 0.8;
    }

    out_color = vec4(gray, gray, gray, 1.0);
}