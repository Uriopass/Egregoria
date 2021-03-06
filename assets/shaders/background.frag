#version 450
#include "render_params.glsl"

layout(location=0) in vec3 in_pos;
layout(location=0) out vec4 out_color;

layout(set = 0, binding = 0) uniform Uni {RenderParams params;};

layout(set = 1, binding = 0) uniform texture2D t_bnoise;
layout(set = 1, binding = 1) uniform sampler s_bnoise;

layout(set = 2, binding = 0) uniform texture2D t_gradientsky;
layout(set = 2, binding = 1) uniform sampler s_gradientsky;

layout(set = 2, binding = 2) uniform texture2D t_starfield;
layout(set = 2, binding = 3) uniform sampler s_starfield;

#define PI 3.141592
#define iSteps 12
#define jSteps 4

vec2 rsi(vec3 r0, vec3 rd, float sr) {
    // ray-sphere intersection that assumes
    // the sphere is centered at the origin.
    // No intersection when result.x > result.y
    float a = dot(rd, rd);
    float b = 2.0 * dot(rd, r0);
    float c = dot(r0, r0) - (sr * sr);
    float d = (b*b) - 4.0*a*c;
    if (d < 0.0) return vec2(1e5,-1e5);
    return vec2(
    (-b - sqrt(d))/(2.0*a),
    (-b + sqrt(d))/(2.0*a)
    );
}

const vec3 r0       = vec3(0,6372e3,0);               // ray origin
const float iSun    = 22.0;                           // intensity of the sun
const float rPlanet = 6371e3;                         // radius of the planet in meters
const float rAtmos  = 6471e3;                         // radius of the atmosphere in meters
const vec3 kRlh     = vec3(5.5e-6, 13.0e-6, 22.4e-6); // Rayleigh scattering coefficient
const float kMie    = 21e-6;                          // Mie scattering coefficient
const float shRlh   = 8e3;                            // Rayleigh scale height
const float shMie   = 1.2e3;                          // Mie scale height
const float g       = 0.758;                          // Mie preferred scattering direction

// r and pSun are normalized
vec3 atmosphere(vec3 r, vec3 pSun) {
    // Calculate the step size of the primary ray.
    vec2 p = rsi(r0, r, rAtmos);
    if (p.x > p.y) return vec3(0,0,0);
    p.y = min(p.y, rsi(r0, r, rPlanet).x);
    float iStepSize = (p.y - p.x) / float(iSteps);

    // Initialize the primary ray time.
    float iTime = iStepSize * 0.375;

    // Initialize accumulators for Rayleigh and Mie scattering.
    vec3 totalRlh = vec3(0,0,0);
    vec3 totalMie = vec3(0,0,0);

    // Initialize optical depth accumulators for the primary ray.
    float iOdRlh = 0.0;
    float iOdMie = 0.0;

    // Calculate the Rayleigh and Mie phases.
    float mu = dot(r, pSun);
    float mumu = mu * mu;
    float gg = g * g;
    float pRlh = 3.0 / (16.0 * PI) * (1.0 + mumu);
    float pMie = 3.0 / (8.0 * PI) * ((1.0 - gg) * (mumu + 1.0)) / (pow(1.0 + gg - 2.0 * mu * g, 1.5) * (2.0 + gg));

    // Sample the primary ray.
    for (int i = 0; i < iSteps; i++) {

        // Calculate the primary ray sample position.
        vec3 iPos = r0 + r * iTime;

        // Calculate the height of the sample.
        float iHeight = length(iPos) - rPlanet;

        // Calculate the optical depth of the Rayleigh and Mie scattering for this step.
        float odStepRlh = exp(-iHeight / shRlh) * iStepSize;
        float odStepMie = exp(-iHeight / shMie) * iStepSize;

        // Accumulate optical depth.
        iOdRlh += odStepRlh;
        iOdMie += odStepMie;

        // Calculate the step size of the secondary ray.
        float jStepSize = rsi(iPos, pSun, rAtmos).y / float(jSteps);

        // Initialize the secondary ray time.
        float jTime = 0.0;

        // Initialize optical depth accumulators for the secondary ray.
        float jOdRlh = 0.0;
        float jOdMie = 0.0;

        // Sample the secondary ray.
        for (int j = 0; j < jSteps; j++) {

            // Calculate the secondary ray sample position.
            vec3 jPos = iPos + pSun * (jTime + jStepSize * 0.5);

            // Calculate the height of the sample.
            float jHeight = length(jPos) - rPlanet;

            // Accumulate the optical depth.
            jOdRlh += exp(-jHeight / shRlh) * jStepSize;
            jOdMie += exp(-jHeight / shMie) * jStepSize;

            // Increment the secondary ray time.
            jTime += jStepSize;
        }

        // Calculate attenuation.
        vec3 attn = exp(-(kMie * (iOdMie + jOdMie) + kRlh * (iOdRlh + jOdRlh)));

        // Accumulate scattering.
        totalRlh += odStepRlh * attn;
        totalMie += odStepMie * attn;

        // Increment the primary ray time.
        iTime += iStepSize;
    }

    // Calculate and return the final color.
    return iSun * (pRlh * kRlh * totalRlh + pMie * kMie * totalMie);
}

float dither() {
    float color = texture(sampler2D(t_bnoise, s_bnoise), gl_FragCoord.xy / 512.0).r;
    return (color - 0.5) / 255.0;
}

float atan2(float y, float x)
{
    bool s = (abs(x) > abs(y));
    return mix(PI/2.0 - atan(x,y), atan(y,x), s);
}

void main()
{
    vec3 fsun = params.sun;
    fsun.yz = fsun.zy;
    vec3 pos = normalize(in_pos.xyz);
    pos.yz = pos.zy;

    float longitude = atan2(pos.x, pos.z);

    vec3 color;
    if (params.realistic_sky != 0) {
        color = atmosphere(
            pos,           // normalized ray direction
            fsun           // normalized sun direction
        );
    } else {
        color = texture(sampler2D(t_gradientsky, s_gradientsky), vec2(0.5 - fsun.y * 0.5, 1.0 - max(0.01, pos.y))).rgb;
    }

    color += max(pos.y + 0.1, 0.0) * 5.0 * texture(sampler2D(t_starfield, s_starfield), vec2(longitude, pos.y)).rgb; // starfield
    color += max(pos.y, 0.0) * 10000.0 * smoothstep(0.99993, 1.0, dot(fsun, pos)); // sun

    // Apply exposure.
    out_color.rgb = 1.0 - exp(-color) + dither();
    out_color.a = 1.0;
}