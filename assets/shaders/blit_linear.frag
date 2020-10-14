#version 450

layout(location=0) in vec2 in_uv;
layout(location=0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D t_color;
layout(set = 0, binding = 1) uniform sampler s_color;

vec4 srgb_to_linear(vec4 srgb) {
    vec3 color_srgb = srgb.rgb;
    vec3 selector = ceil(color_srgb - 0.04045); // 0 if under value, 1 if over
    vec3 under = color_srgb / 12.92;
    vec3 over = pow((color_srgb + 0.055) / 1.055, vec3(2.4));
    vec3 result = mix(under, over, selector);
    return vec4(result, srgb.a);
}

void main() {
    out_color = srgb_to_linear(texture(sampler2D(t_color, s_color), in_uv));
}
