#version 450
#include "light_params.glsl"

layout(location=0) in vec2 in_uv;
layout(location=1) in vec2 in_wv;

layout(location=0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D t_light;
layout(set = 0, binding = 1) uniform sampler s_light;

layout(set = 0, binding = 2) uniform texture2D t_color;
layout(set = 0, binding = 3) uniform sampler s_color;

layout(set = 0, binding = 4) uniform texture2D t_noise;
layout(set = 0, binding = 5) uniform sampler s_noise;

layout(set = 0, binding = 6) uniform texture2D t_blue_noise;
layout(set = 0, binding = 7) uniform sampler s_blue_noise;

layout(set = 1, binding = 0) uniform Uni {LightParams params;};

float tnoise( vec2 v) {
    return texture(sampler2D(t_noise, s_noise), v * 0.1).r * 2.0 - 0.5;
}

const float FBM_MAG = 0.4;

float dither() {
    float color = texture(sampler2D(t_blue_noise, s_blue_noise), gl_FragCoord.xy / 512.0).r;
    return color / 255.0;
}

void main() {
    float street_light = clamp(texture(sampler2D(t_light, s_light), in_uv).r, 0.0, 1.0);
    vec3 color = texture(sampler2D(t_color, s_color), in_uv).rgb;

    float sun_mult =  clamp(1.2 * params.sun.z, 0.1, 1.0);

    vec3 real_ambiant = vec3(sun_mult);

    vec3 yellow =  street_light * vec3(0.9, 0.9, 0.7);
    out_color   =  vec4((color + dither()) * clamp(real_ambiant + yellow, 0.0, 1.0), 1.0);
}
