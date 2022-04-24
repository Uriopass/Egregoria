struct FragmentOutput {
    [[location(0)]] out_color: vec4<f32>;
};

var<private> in_uv_1: vec2<f32>;
var<private> out_color: vec4<f32>;

fn main_1() {
    var v: f32;
    var strength: f32;

    let _e8: vec2<f32> = in_uv_1;
    let _e9: vec2<f32> = in_uv_1;
    v = (0.09000000357627869 + dot(_e8, _e9));
    let _e13: f32 = v;
    let _e14: f32 = v;
    strength = ((0.008100000210106373 / ((_e13 * _e14) * (1.0 - 0.006817608077564465))) - 0.006817608077564465);
    let _e26: f32 = strength;
    out_color.x = clamp(_e26, 0.0, 1.0);
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] in_uv: vec2<f32>) -> FragmentOutput {
    in_uv_1 = in_uv;
    main_1();
    let _e24: vec4<f32> = out_color;
    return FragmentOutput(_e24);
}
