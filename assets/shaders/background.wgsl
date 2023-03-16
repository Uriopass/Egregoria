#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: RenderParams;

@group(1) @binding(0) var t_bnoise: texture_2d<f32>;
@group(1) @binding(1) var s_bnoise: sampler;

@group(2) @binding(0) var t_gradientsky: texture_2d<f32>;
@group(2) @binding(1) var s_gradientsky: sampler;
@group(2) @binding(2) var t_starfield: texture_2d<f32>;
@group(2) @binding(3) var s_starfield: sampler;
@group(2) @binding(4) var t_environment: texture_cube<f32>;
@group(2) @binding(5) var s_environment: sampler;

#include "dither.wgsl"

fn rsi(r0: vec3<f32>, rd: vec3<f32>, sr: f32) -> vec2<f32> {
    // ray-sphere intersection that assumes
    // the sphere is centered at the origin.
    // No intersection when result.x > result.y
    let a: f32 = dot(rd, rd);
    let b: f32 = 2.0 * dot(rd, r0);
    let c: f32 = dot(r0, r0) - (sr * sr);
    let d: f32 = (b*b) - 4.0*a*c;
    if (d < 0.0) {
        return vec2(1e5,-1e5);
    }
    return vec2(
    (-b - sqrt(d))/(2.0*a),
    (-b + sqrt(d))/(2.0*a)
    );
}

const PI: f32 = 3.141592;
const iSteps: i32 = 12;
const jSteps: i32 = 4;

const r0: vec3<f32>    = vec3<f32>(0.0,6372000.0,0.0);          // ray origin
const iSun: f32        = 22.0;                           // intensity of the sun
const rPlanet: f32     = 6371e3;                         // radius of the planet in meters
const rAtmos: f32      = 6471e3;                         // radius of the atmosphere in meters
const kRlh: vec3<f32>  = vec3<f32>(5.5e-6, 13.0e-6, 22.4e-6); // Rayleigh scattering coefficient
const kMie: f32        = 21e-6;                          // Mie scattering coefficient
const shRlh: f32       = 8e3;                            // Rayleigh scale height
const shMie: f32       = 1.2e3;                          // Mie scale height
const g: f32           = 0.758;                          // Mie preferred scattering direction

// r and pSun are normalized
fn atmosphere(r: vec3<f32>, pSun: vec3<f32>) -> vec3<f32> {
    // Calculate the step size of the primary ray.
    var p: vec2<f32> = rsi(r0, r, rAtmos);
    if (p.x > p.y) {
        return vec3(0.0,0.0,0.0);
    }
    p.y = min(p.y, rsi(r0, r, rPlanet).x);
    let iStepSize: f32 = (p.y - p.x) / f32(iSteps);

    // Initialize the primary ray time.
    var iTime: f32 = iStepSize * 0.375;

    // Initialize accumulators for Rayleigh and Mie scattering.
    var totalRlh: vec3<f32> = vec3(0.0,0.0,0.0);
    var totalMie: vec3<f32> = vec3(0.0,0.0,0.0);

    // Initialize optical depth accumulators for the primary ray.
    var iOdRlh: f32 = 0.0;
    var iOdMie: f32 = 0.0;

    // Calculate the Rayleigh and Mie phases.
    let mu: f32 = dot(r, pSun);
    let mumu: f32 = mu * mu;
    let gg: f32 = g * g;
    let pRlh: f32 = 3.0 / (16.0 * PI) * (1.0 + mumu);
    let pMie: f32 = 3.0 / (8.0 * PI) * ((1.0 - gg) * (mumu + 1.0)) / (pow(1.0 + gg - 2.0 * mu * g, 1.5) * (2.0 + gg));

    // Sample the primary ray.
    for (var i: i32 = 0; i < iSteps; i++) {

        // Calculate the primary ray sample position.
        let iPos: vec3<f32> = r0 + r * iTime;

        // Calculate the height of the sample.
        let iHeight: f32 = length(iPos) - rPlanet;

        // Calculate the optical depth of the Rayleigh and Mie scattering for this step.
        let odStepRlh: f32 = exp(-iHeight / shRlh) * iStepSize;
        let odStepMie: f32 = exp(-iHeight / shMie) * iStepSize;

        // Accumulate optical depth.
        iOdRlh += odStepRlh;
        iOdMie += odStepMie;

        // Calculate the step size of the secondary ray.
        let jStepSize: f32 = rsi(iPos, pSun, rAtmos).y / f32(jSteps);

        // Initialize the secondary ray time.
        var jTime: f32 = 0.0;

        // Initialize optical depth accumulators for the secondary ray.
        var jOdRlh: f32 = 0.0;
        var jOdMie: f32 = 0.0;

        // Sample the secondary ray.
        for (var j: i32 = 0; j < jSteps; j++) {

            // Calculate the secondary ray sample position.
            let jPos: vec3<f32> = iPos + pSun * (jTime + jStepSize * 0.5);

            // Calculate the height of the sample.
            let jHeight: f32 = length(jPos) - rPlanet;

            // Accumulate the optical depth.
            jOdRlh += exp(-jHeight / shRlh) * jStepSize;
            jOdMie += exp(-jHeight / shMie) * jStepSize;

            // Increment the secondary ray time.
            jTime += jStepSize;
        }

        // Calculate attenuation.
        let attn: vec3<f32> = exp(-(kMie * (iOdMie + jOdMie) + kRlh * (iOdRlh + jOdRlh)));

        // Accumulate scattering.
        totalRlh += odStepRlh * attn;
        totalMie += odStepMie * attn;

        // Increment the primary ray time.
        iTime += iStepSize;
    }

    // Calculate and return the final color.
    return iSun * (pRlh * kRlh * totalRlh + pMie * kMie * totalMie);
}

fn atan2(y: f32, x: f32) -> f32
{
    let s: bool = (abs(x) > abs(y));
    return select(PI/2.0 - atan(y/x), atan(y/x), s);
}

@fragment
fn frag(@location(0) in_pos: vec3<f32>, @builtin(position) position: vec4<f32>) -> FragmentOutput {
    var fsun: vec3<f32> = params.sun;
    fsun = vec3(fsun.x, fsun.z, fsun.y);
    var pos: vec3<f32> = normalize(in_pos.xyz);
    pos = vec3(pos.x, pos.z, pos.y);

    let longitude: f32 = atan2(pos.x, pos.z);

    var color: vec3<f32>;
    if (params.realistic_sky != 0) {
        color = atmosphere(
            pos,           // normalized ray direction
            fsun           // normalized sun direction
        );
    } else {
        color = textureSample(t_gradientsky, s_gradientsky, vec2(0.5 - fsun.y * 0.5, 1.0 - max(0.01, pos.y))).rgb;
    }

    color = textureSample(t_environment, s_environment, vec3(pos.x, pos.z, pos.y)).rgb;

    color = color + max(pos.y + 0.1, 0.0) * 5.0 * textureSample(t_starfield, s_starfield, vec2(longitude, pos.y)).rgb; // starfield
    color = color + max(pos.y, 0.0) * 10000.0 * smoothstep(0.99993, 1.0, dot(fsun, pos)); // sun

    // Apply exposure.
    let ocrgb = 1.0 - exp(-color) + dither(position.xy);
    return FragmentOutput(vec4(ocrgb.r, ocrgb.g, ocrgb.b, 1.0));
}

struct VertexOutput {
    @location(0) out_pos: vec3<f32>,
    @builtin(position) member: vec4<f32>,
}

@vertex
fn vert(@location(0) in_pos: vec3<f32>, @location(1) in_uv: vec2<f32>) -> VertexOutput {
    let near: vec4<f32> = (params.invproj * vec4(in_pos.xy, -1.0, 1.0));
    let far: vec4<f32> = (params.invproj * vec4(in_pos.xy, 1.0, 1.0));
    let out_pos = near.xyz * far.w - far.xyz * near.w;
    return VertexOutput(out_pos, vec4(in_pos.xy, 0.0, 1.0));
}