#version 450

layout(location=0) in vec2 in_uv;
layout(location=1) in vec2 in_wv;
layout(location=2) in float in_zoom;
layout(location=3) in float time;

layout(location=0) out vec4 out_color;

vec3 permute(vec3 x) { return mod(((x*34.0)+1.0)*x, 289.0); }

float snoise(vec2 v){
    const vec4 C = vec4(0.211324865405187, 0.366025403784439,
    -0.577350269189626, 0.024390243902439);
    vec2 i  = floor(v + dot(v, C.yy));
    vec2 x0 = v -   i + dot(i, C.xx);
    vec2 i1;
    i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    vec4 x12 = x0.xyxy + C.xxzz;
    x12.xy -= i1;
    i = mod(i, 289.0);
    vec3 p = permute(permute(i.y + vec3(0.0, i1.y, 1.0))
    + i.x + vec3(0.0, i1.x, 1.0));
    vec3 m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy),
    dot(x12.zw, x12.zw)), 0.0);
    m = m*m;
    m = m*m;
    vec3 x = 2.0 * fract(p * C.www) - 1.0;
    vec3 h = abs(x) - 0.5;
    vec3 ox = floor(x + 0.5);
    vec3 a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * (a0*a0 + h*h);
    vec3 g;
    g.x  = a0.x  * x0.x  + h.x  * x0.y;
    g.yz = a0.yz * x12.xz + h.yz * x12.yw;
    return 130.0 * dot(m, g);
}

const float FBM_MAG = 0.4;

float fnoise(float ampl) {
    vec2 dec = 0.1 + in_wv.xy * ampl;

    float noise = 0.0;
    float amplitude = 0.6;
    float freq = 0.0;

    for (int i = 0; i < 8; i++) {
        noise += amplitude * (snoise(dec) + 1.0) * FBM_MAG;
        dec *= 1.0 / FBM_MAG;
        amplitude *= FBM_MAG;
    }

    return noise;
}

float disturbed_noise(float noise) {
    float noise2 = fnoise(0.007);

    float zoom = clamp(log(in_zoom) * 0.01 + 0.1, 0.0, 0.2);

    return noise * (1.0 - zoom) + noise2 * zoom;
}

void main() {
    float noise = fnoise(0.00002);

    noise -= length(in_wv - vec2(0.0, 10000.0)) * 0.00002;
    noise = max(noise, 0);

    float dnoise = disturbed_noise(noise);

    if (noise < 0.05) { // deep water
        out_color = vec4(0.02, 0.02, 0.5 - noise, 1.0);
    } else if (noise < 0.09 + sin(time * 0.1) * 0.01) { // shallow water
        out_color = vec4(0.1, 0.3, 0.7 - dnoise, 1.0);
    } else if (noise < 0.12) { // sand
        out_color = (1.0 - dnoise) * vec4(0.95, 0.9, 0.3, 1.0);
    } else if (noise < 0.6) { // grass
        out_color = vec4(dnoise * 0.2, dnoise, dnoise * 0.1, 1.0);
    } else { // mountain
        noise -= 0.1;
        noise *= 1.4;
        out_color = vec4(dnoise, dnoise, dnoise, 1.0);
    }
}