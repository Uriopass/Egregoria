struct FragmentOutput {
    @location(0) out_cube: vec4<f32>,
}

@group(0) @binding(0) var t_environment: texture_cube<f32>;
@group(0) @binding(1) var s_environment: sampler;

const PI: f32 = 3.141592653589793238462;
const STEP: f32 = 0.02;

@fragment
fn frag(@location(0) wpos: vec3<f32>) -> FragmentOutput {
    let normal: vec3<f32> = normalize(wpos);

    var irradiance: vec3<f32> = vec3(0.0);
    let right: vec3<f32> = normalize(cross(vec3(0.0, 0.0, 1.0), normal));
    let up:    vec3<f32> = normalize(cross(normal, right));

    for(var phi: f32 = 0.0; phi < 2.0 * PI; phi += STEP)
    {
        for(var theta: f32 = 0.0; theta < 0.5 * PI; theta += STEP)
        {
            let ts: vec3<f32> = vec3(sin(theta) * cos(phi),  sin(theta) * sin(phi), cos(theta)); // tangeant space
            let ws: vec3<f32> = ts.x * right + ts.y * up + ts.z * normal; // world space

            irradiance += min(vec3(10.0), textureSample(t_environment, s_environment, ws).rgb) * cos(theta) * sin(theta);
        }
    }
    let n_samples = (1 + i32(2.0 * PI / STEP)) * (1 + i32(0.5 * PI / STEP));
    irradiance = (PI * irradiance) / f32(n_samples);

    return FragmentOutput(vec4(irradiance.r, irradiance.g, irradiance.b, 1.0));
}