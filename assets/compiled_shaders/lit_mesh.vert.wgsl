struct Uniforms {
    u_view_proj: mat4x4<f32>;
};

struct VertexOutput {
    [[location(0)]] out_color: vec4<f32>;
    [[location(1)]] out_normal: vec3<f32>;
    [[location(2)]] out_wpos: vec3<f32>;
    [[location(3)]] out_uv: vec2<f32>;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> in_position_1: vec3<f32>;
var<private> in_normal_1: vec3<f32>;
var<private> in_uv_1: vec2<f32>;
var<private> in_color_1: vec4<f32>;
var<private> out_color: vec4<f32>;
var<private> out_normal: vec3<f32>;
var<private> out_wpos: vec3<f32>;
var<private> out_uv: vec2<f32>;
[[group(0), binding(0)]]
var<uniform> global: Uniforms;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    let _e10: vec3<f32> = in_position_1;
    out_wpos = _e10;
    let _e11: vec4<f32> = in_color_1;
    out_color = _e11;
    let _e12: vec3<f32> = in_normal_1;
    out_normal = _e12;
    let _e13: vec2<f32> = in_uv_1;
    out_uv = _e13;
    let _e15: mat4x4<f32> = global.u_view_proj;
    let _e16: vec3<f32> = in_position_1;
    gl_Position = (_e15 * vec4<f32>(_e16.x, _e16.y, _e16.z, 1.0));
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] in_position: vec3<f32>, [[location(1)]] in_normal: vec3<f32>, [[location(2)]] in_uv: vec2<f32>, [[location(3)]] in_color: vec4<f32>) -> VertexOutput {
    in_position_1 = in_position;
    in_normal_1 = in_normal;
    in_uv_1 = in_uv;
    in_color_1 = in_color;
    main_1();
    let _e27: vec4<f32> = out_color;
    let _e29: vec3<f32> = out_normal;
    let _e31: vec3<f32> = out_wpos;
    let _e33: vec2<f32> = out_uv;
    let _e35: vec4<f32> = gl_Position;
    return VertexOutput(_e27, _e29, _e31, _e33, _e35);
}
