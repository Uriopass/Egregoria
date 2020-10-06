#version 450

layout(location=0) in vec2 in_uv;
layout(location=0) out vec4 out_color;

layout(set = 0, binding = 0) uniform texture2D t_light;
layout(set = 0, binding = 1) uniform sampler s_light;

layout(set = 1, binding = 0) uniform texture2D t_color;
layout(set = 1, binding = 1) uniform sampler s_color;

layout(set = 2, binding = 0) uniform LightParams {
    vec3 ambiant;
    float time;
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

void main() {

    float light = clamp(texture(sampler2D(t_light, s_light), in_uv).r, 0.0, 1.0);
    vec4  color = vec4(texture(sampler2D(t_color, s_color), in_uv).rgb, 1.0);

    float randv = random(floatBitsToUint(vec3(in_uv, time))) / 256.0;

    vec3 yellow =  light * (vec3(0.9, 0.9, 0.7) + randv);
    out_color   =  (color + randv) * vec4(ambiant + yellow, 1.0);
}
