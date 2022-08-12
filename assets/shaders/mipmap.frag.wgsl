struct FragmentOutput {
    @location(0) o_Target: vec4<f32>,
}

@group(0) @binding(0) var t_Color: texture_2d<f32>;
@group(0) @binding(1) var s_Color: sampler;

@fragment 
fn main(@location(0) v_TexCoord: vec2<f32>) -> FragmentOutput {
    let o_Target = textureSampleLevel(t_Color, s_Color, v_TexCoord, 0.0);
    return FragmentOutput(o_Target);
}
