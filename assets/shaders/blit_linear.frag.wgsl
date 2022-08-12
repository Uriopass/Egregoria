struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

@group(0) @binding(0) var t_color: texture_2d<f32>;
@group(0) @binding(1) var s_color: sampler;

fn srgb_to_linear(srgb: vec4<f32>) -> vec4<f32> {
    let color_srgb: vec3<f32> = srgb.rgb;
    let selector: vec3<f32> = ceil(color_srgb - 0.04045); // 0 if under value, 1 if over
    let under: vec3<f32> = color_srgb / 12.92;
    let over: vec3<f32> = pow((color_srgb + 0.055) / 1.055, vec3(2.4));
    let result: vec3<f32> = mix(under, over, selector);
    return vec4(result, srgb.a);
}


@fragment 
fn main(@location(0) in_uv: vec2<f32>) -> FragmentOutput {
    let out_color = srgb_to_linear(textureSample(t_color, s_color, in_uv));
    return FragmentOutput(out_color);
}
