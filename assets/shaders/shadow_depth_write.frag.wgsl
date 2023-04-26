struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
}

@fragment
fn frag(@location(0) in_tint: vec4<f32>,
        @location(1) in_normal: vec3<f32>,
        @location(2) in_tangent: vec4<f32>,
        @location(3) in_wpos: vec3<f32>,
        @location(4) in_uv: vec2<f32>,
        @builtin(position) position: vec4<f32>) -> FragmentOutput {
    return FragmentOutput(position.z);
}