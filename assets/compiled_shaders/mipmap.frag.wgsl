struct FragmentOutput {
    [[location(0)]] o_Target: vec4<f32>;
};

var<private> v_TexCoord_1: vec2<f32>;
var<private> o_Target: vec4<f32>;
[[group(0), binding(0)]]
var t_Color: texture_2d<f32>;
[[group(0), binding(1)]]
var s_Color: sampler;

fn main_1() {
    let _e6: vec2<f32> = v_TexCoord_1;
    let _e8: vec4<f32> = textureSampleLevel(t_Color, s_Color, _e6, 0.0);
    o_Target = _e8;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] v_TexCoord: vec2<f32>) -> FragmentOutput {
    v_TexCoord_1 = v_TexCoord;
    main_1();
    let _e11: vec4<f32> = o_Target;
    return FragmentOutput(_e11);
}
