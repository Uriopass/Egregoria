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

// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
uint hash( uint x ) {
    x += ( x << 10u );
    x ^= ( x >>  6u );
    x += ( x <<  3u );
    x ^= ( x >> 11u );
    x += ( x << 15u );
    return x;
}

// Construct a float with half-open range [0:1] using low 23 bits.
// All zeroes yields 0.0, all ones yields the next smallest representable value below 1.0.
float floatConstruct( uint m ) {
    const uint ieeeMantissa = 0x007FFFFFu; // binary32 mantissa bitmask
    const uint ieeeOne      = 0x3F800000u; // 1.0 in IEEE binary32

    m &= ieeeMantissa;                     // Keep only mantissa bits (fractional part)
    m |= ieeeOne;                          // Add fractional part to 1.0

    float  f = uintBitsToFloat( m );       // Range [1:2]
    return f - 1.0;                        // Range [0:1]
}
// Pseudo-random value in half-open range [0:1].
float random( uvec3 v ) { return floatConstruct(hash(v.x ^ hash(v.y) ^ hash(v.z))); }

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

void main() {
    float street_light = clamp(texture(sampler2D(t_light, s_light), in_uv).r, 0.0, 1.0);
    vec3 color = texture(sampler2D(t_color, s_color), in_uv).rgb;

    float cloud = cloud(in_wv.xy + time * 100.0, 0.0001);

    float sun_mult =  clamp(1.2 * sun.z, 0.1, 1.0);

    color += mix(0.0, cloud, clamp((height - 4000.0) * 0.0001, 0.0, 1.0));

    vec3 real_ambiant = vec3(sun_mult);

    float randv = random(floatBitsToUint(vec3(in_uv, time))) / 256.0;

    vec3 yellow =  (street_light + randv) * vec3(0.9, 0.9, 0.7);
    out_color   =  vec4((color + randv) * clamp(real_ambiant + yellow, 0.0, 1.0), 1.0);
}
