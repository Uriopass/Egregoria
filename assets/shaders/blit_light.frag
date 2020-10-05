#version 450

layout(location=0) in vec2 in_uv;

layout(location=0) out vec4 out_color;

const float LIGHT_HEIGHT = 0.1;
const float LIGHT_DECAY = LIGHT_HEIGHT / sqrt(LIGHT_HEIGHT * LIGHT_HEIGHT + 1.0);

void main() {
    vec2 d = in_uv;
    float strength = LIGHT_HEIGHT / sqrt(LIGHT_HEIGHT * LIGHT_HEIGHT + dot(d, d)) - LIGHT_DECAY;

    out_color.r = clamp(strength, 0.0, 1.0);
}