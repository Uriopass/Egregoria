#version 450

layout(location=0) in vec2 in_uv;
layout(location=1) in vec2 in_wv;
layout(location=2) in vec3 sun;

layout(location=0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D t_light;
layout(set = 0, binding = 1) uniform sampler s_light;

layout(set = 0, binding = 2) uniform texture2D t_color;
layout(set = 0, binding = 3) uniform sampler s_color;

layout(set = 0, binding = 4) uniform texture2D t_noise;
layout(set = 0, binding = 5) uniform sampler s_noise;

layout(set = 1, binding = 0) uniform LightParams {
    mat4 invproj;
    vec4 ambiant;
    float time;
    float height;
};

float tnoise( vec2 v) {
    return texture(sampler2D(t_noise, s_noise), v * 0.1).r * 2.0 - 0.5;
}

const float FBM_MAG = 0.4;

float cloud(vec2 pos, float ampl) {
    vec2 dec = pos * ampl;

    float noise = 0.0;
    float amplitude = 0.7;

    for (int i = 0; i < 2; i++) {
        float v = tnoise(dec);
        noise += amplitude * v;

        dec *= 1.0 / FBM_MAG;
        amplitude *= FBM_MAG;
    }

    return 0.1 * clamp(noise - 0.55, 0.0, 1.0);
}

const int ditheringMat[16] = int[](0,  8,  2,  10,
    12, 4,  14, 6,
    3,  11, 1,  9,
    15, 7,  13, 5);

const float IDX_DIV = 256.0 * 16.0;

float indexValue() {
    int x = int(mod(gl_FragCoord.x, 4));
    int y = int(mod(gl_FragCoord.y, 4));
    return ditheringMat[(x + y * 4)];
}

void main() {
    float street_light = clamp(texture(sampler2D(t_light, s_light), in_uv).r, 0.0, 1.0);
    vec3 color = texture(sampler2D(t_color, s_color), in_uv).rgb;

    float cloud = cloud(in_wv.xy + time * 100.0, 0.0001);

    float sun_mult =  clamp(1.2 * sun.z, 0.1, 1.0);

    color += mix(0.0, cloud, clamp((height - 4000.0) * 0.0001, 0.0, 1.0));

    vec3 real_ambiant = vec3(sun_mult);

    float v = indexValue() / IDX_DIV;

    vec3 yellow =  street_light * vec3(0.9, 0.9, 0.7);
    out_color   =  vec4((color + v) * clamp(real_ambiant + yellow, 0.0, 1.0), 1.0);
}
