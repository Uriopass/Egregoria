@group(1) @binding(0) var t_albedo: texture_2d<f32>;
@group(1) @binding(1) var s_albedo: sampler;

@fragment
fn frag(@location(0) in_tint: vec4<f32>,
        @location(1) in_normal: vec3<f32>,
        @location(2) in_tangent: vec4<f32>,
        @location(3) in_wpos: vec3<f32>,
        @location(4) in_uv: vec2<f32>,
        @builtin(position) position: vec4<f32>) {
    let alpha: f32 = textureSample(t_albedo, s_albedo, in_uv).a;

    if (alpha < 0.5) {
        discard;
    }

    return;
}
