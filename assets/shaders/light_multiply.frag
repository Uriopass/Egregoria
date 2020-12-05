#version 450

layout(location=0) in vec2 in_uv;
layout(location=1) in vec2 in_wv;
layout(location=2) in vec3 sun;

layout(location=0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D t_light;
layout(set = 0, binding = 1) uniform sampler s_light;

layout(set = 1, binding = 0) uniform texture2D t_color;
layout(set = 1, binding = 1) uniform sampler s_color;

layout(set = 2, binding = 0) uniform texture2D t_normal;
layout(set = 2, binding = 1) uniform sampler s_normal;

layout(set = 3, binding = 0) uniform LightParams {
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
float random( float x ) { return floatConstruct(hash(floatBitsToUint(x))); }

// Pseudo-random value in half-open range [0:1].
float random( uvec3 v ) { return floatConstruct(hash(v.x ^ hash(v.y) ^ hash(v.z))); }

float permute(float x) {
    return mod((34.0 * x + 1.0)*x, 289.0);
}

// Gradient mapping with an extra rotation.
vec2 grad2(vec2 p, float rot) {
    // Map from a line to a diamond such that a shift maps to a rotation.
    float u = permute(permute(p.x) + p.y) * 0.0243902439 + rot;// Rotate by shift
    u = 4.0 * fract(u) - 2.0;
    return vec2(abs(u)-1.0, abs(abs(u+1.0)-2.0)-1.0);
}

float srdnoise(vec2 v, float rot) {
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
    vec2 g0 = grad2(i, rot);
    vec2 g1 = grad2(i + i1, rot);
    vec2 g2 = grad2(i + 1.0, rot);

    // Compute noise contributions from each corner
    vec3 gv = vec3(dot(g0, x0), dot(g1, v1), dot(g2, v2));// ramp: g dot v

    // Add contributions from the three corners and return
    return 103.0 * dot(t4, gv);
}

const float FBM_MAG = 0.4;

float cloud(vec2 pos, float ampl) {
    vec2 dec = pos * ampl;

    float noise = 0.0;
    float amplitude = 0.7;

    for (int i = 0; i < 4; i++) {
        float v = srdnoise(dec, time * 0.0002);
        noise += amplitude * v;

        dec *= 1.0 / FBM_MAG;
        amplitude *= FBM_MAG;
    }

    return 0.1 * clamp(noise - 0.4, 0.0, 1.0);
}

void main() {
    float street_light = clamp(texture(sampler2D(t_light, s_light), in_uv).r, 0.0, 1.0);
    vec3  color = texture(sampler2D(t_color, s_color), in_uv).rgb;
    vec3  normal = texture(sampler2D(t_normal, s_normal), in_uv).xyz;
    float cloud = cloud(in_wv.xy + time * 100.0, 0.0001);

    float sun_mult =  clamp(1.2 * dot(normal, sun), 0.1, 1.0);

    color += mix(0.0, cloud, min(1.0, height * 0.0005 - 0.3));

    vec3 real_ambiant = vec3(sun_mult);

    float randv = random(floatBitsToUint(vec3(in_uv, time))) / 256.0;

    vec3 yellow =  (street_light + randv) * vec3(0.9, 0.9, 0.7);
    out_color   =  vec4((color + randv) * clamp(real_ambiant + yellow, 0.0, 1.0), 1.0);
}
