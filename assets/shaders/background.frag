#version 450

layout(location=0) in vec2 in_uv;
layout(location=1) in vec2 in_wv;
layout(location=2) in float in_zoom;

layout(location=0) out vec4 out_color;
layout(location=1) out vec4 out_normal;

layout(set = 1, binding = 0) uniform BGParams {
    vec4 sea_color;
    vec4 grass_color;
    vec4 sand_color;
    float time;
};

layout(set = 2, binding = 0) uniform texture2D t_noise;
layout(set = 2, binding = 1) uniform sampler s_noise;

float permute(float x) {
    return mod((34.0 * x + 1.0)*x, 289.0);
}

// Gradient mapping with an extra rotation.
vec2 grad2(vec2 p) {
    // Map from a line to a diamond such that a shift maps to a rotation.
    float u = permute(permute(p.x) + p.y) * 0.0243902439;
    u = 4.0 * fract(u) - 2.0;
    return vec2(abs(u)-1.0, abs(abs(u+1.0)-2.0)-1.0);
}

float srdnoise(vec2 v) {
    const vec3 C = vec3(0.211324865405187, 0.366025403784439,
    -0.577350269189626);

    vec2 i = floor(v + dot(v, C.yy));
    vec2 x0 = v - i + dot(i, C.xx);

    vec2 i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2 (0.0, 1.0);

    // Determine the offsets for the other two corners
    vec2 v1 = x0 - i1 + C.x;
    vec2 v2 = x0 - 1.0 + 2.0 * C.x;

    // Wrap coordinates at 289 to avoid float precision problems
    i = mod(i, 289.0);

    // Calculate the circularly symmetric part of each noise wiggle
    vec3 t = max(0.5 - vec3(dot(x0, x0), dot(v1, v1), dot(v2, v2)), 0.0);
    vec3 t2 = t*t;
    vec3 t4 = t2*t2;

    // Calculate the gradients for the three corners
    vec2 g0 = grad2(i);
    vec2 g1 = grad2(i + i1);
    vec2 g2 = grad2(i + 1.0);

    // Compute noise contributions from each corner
    vec3 gv = vec3(dot(g0, x0), dot(g1, v1), dot(g2, v2));// ramp: g dot v
/*
    // Compute partial derivatives in x and y
    vec3 temp = t2 * t * gv;
    grad.x = -8.0 * dot(temp, vec3(x0.x, v1.x, v2.x));
    grad.y = -8.0 * dot(temp, vec3(x0.y, v1.y, v2.y));
    grad.x += dot(t4, vec3(g0.x, g1.x, g2.x));
    grad.y += dot(t4, vec3(g0.y, g1.y, g2.y));
    grad *= 40.0;
*/
    // Add contributions from the three corners and return
    return 40.0 * dot(t4, gv);
}

const float FBM_MAG = 0.4;

float fnoise(vec2 pos, float ampl) {
    vec2 dec = 70.69 + pos * ampl;

    float noise = 0.0;
    float amplitude = 1.0;

    for (int i = 0; i < 5; i++) {
        float v = srdnoise(dec, 0.0);
        noise += amplitude * v;

        dec *= 1.0 / FBM_MAG;
        amplitude *= FBM_MAG;
    }

    return noise;
}

float tnoise(vec2 pos) {
    return texture(sampler2D(t_noise, s_noise), pos).r;
}

float disturbed_noise(vec2 pos, float noise) {
    float noise2 = tnoise(pos * 0.0005) * 3.0;

    float zoom = clamp(log(in_zoom) * 0.01 + 0.2, 0.0, 1.0);

    return noise * (1.0 - zoom) + noise2 * zoom;
}

void main() {
    float noise = fnoise(in_wv, 0.00003) + 0.2;

    noise = clamp(noise, 0.0, 1.0);

    if (noise < 0.1) { // deep water
        float dnoise = disturbed_noise(in_wv, noise);
        out_color =  (1.0 - 0.4 * dnoise + 3.0 * noise) * sea_color;
    } else if (noise < 0.11) { // sand
        out_color = sand_color;
    } else {
        float dnoise = disturbed_noise(in_wv * 3.0, noise);

        out_color = (0.4 + noise * 0.5 + (dnoise - noise) * 0.6) * grass_color;
    }

    out_color.a = 1.0;

    out_normal = vec4(0.0, 0.0, 1.0, 1.0);
}