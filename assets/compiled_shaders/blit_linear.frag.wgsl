struct FragmentOutput {
    [[location(0)]] out_color: vec4<f32>;
};

var<private> in_uv_1: vec2<f32>;
var<private> out_color: vec4<f32>;
[[group(0), binding(0)]]
var t_color: texture_2d<f32>;
[[group(0), binding(1)]]
var s_color: sampler;

fn srgb_to_linear(srgb: vec4<f32>) -> vec4<f32> {
    var srgb_1: vec4<f32>;
    var color_srgb: vec3<f32>;
    var selector: vec3<f32>;
    var under: vec3<f32>;
    var over: vec3<f32>;
    var result: vec3<f32>;

    srgb_1 = srgb;
    let _e6: vec4<f32> = srgb_1;
    color_srgb = _e6.xyz;
    let _e9: vec3<f32> = color_srgb;
    let _e13: vec3<f32> = color_srgb;
    selector = ceil((_e13 - vec3<f32>(0.040449999272823334)));
    let _e19: vec3<f32> = color_srgb;
    under = (_e19 / vec3<f32>(12.920000076293945));
    let _e24: vec3<f32> = color_srgb;
    let _e33: vec3<f32> = color_srgb;
    over = pow(((_e33 + vec3<f32>(0.054999999701976776)) / vec3<f32>(1.0549999475479126)), vec3<f32>(2.4000000953674316));
    let _e47: vec3<f32> = under;
    let _e48: vec3<f32> = over;
    let _e49: vec3<f32> = selector;
    result = mix(_e47, _e48, _e49);
    let _e52: vec3<f32> = result;
    let _e53: vec4<f32> = srgb_1;
    return vec4<f32>(_e52.x, _e52.y, _e52.z, _e53.w);
}

fn main_1() {
    let _e5: vec2<f32> = in_uv_1;
    let _e6: vec4<f32> = textureSample(t_color, s_color, _e5);
    let _e8: vec2<f32> = in_uv_1;
    let _e9: vec4<f32> = textureSample(t_color, s_color, _e8);
    let _e10: vec4<f32> = srgb_to_linear(_e9);
    out_color = _e10;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] in_uv: vec2<f32>) -> FragmentOutput {
    in_uv_1 = in_uv;
    main_1();
    let _e11: vec4<f32> = out_color;
    return FragmentOutput(_e11);
}
