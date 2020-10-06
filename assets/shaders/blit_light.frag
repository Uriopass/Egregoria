#version 450

layout(location=0) in vec2 in_uv;

layout(location=0) out vec4 out_color;

const float H = 0.3;
const float H2 = 0.09;
const float H4 = 0.0081;
const float DECAY = H4 / ((H2 + 1.0) * (H2 + 1.0));

void main() {
    float v = H2 + dot(in_uv, in_uv);
    float strength = H4 / (v * v * (1.0 - DECAY)) - DECAY;

    out_color.r = clamp(strength, 0.0, 1.0);
}