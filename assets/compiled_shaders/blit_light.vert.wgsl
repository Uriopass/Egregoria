struct Proj {
    view_proj: mat4x4<f32>;
};

struct VertexOutput {
    [[location(0)]] out_uv: vec2<f32>;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> in_pos_1: vec3<f32>;
var<private> in_uv_1: vec2<f32>;
var<private> in_instance_pos_1: vec3<f32>;
var<private> in_instance_scale_1: f32;
var<private> out_uv: vec2<f32>;
[[group(0), binding(0)]]
var<uniform> global: Proj;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    let _e8: mat4x4<f32> = global.view_proj;
    let _e9: vec3<f32> = in_pos_1;
    let _e10: f32 = in_instance_scale_1;
    let _e12: vec3<f32> = in_instance_pos_1;
    let _e13: vec3<f32> = ((_e9 * _e10) + _e12);
    gl_Position = (_e8 * vec4<f32>(_e13.x, _e13.y, _e13.z, 1.0));
    let _e20: vec2<f32> = in_uv_1;
    out_uv = _e20;
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] in_pos: vec3<f32>, [[location(1)]] in_uv: vec2<f32>, [[location(2)]] in_instance_pos: vec3<f32>, [[location(3)]] in_instance_scale: f32) -> VertexOutput {
    in_pos_1 = in_pos;
    in_uv_1 = in_uv;
    in_instance_pos_1 = in_instance_pos;
    in_instance_scale_1 = in_instance_scale;
    main_1();
    let _e21: vec2<f32> = out_uv;
    let _e23: vec4<f32> = gl_Position;
    return VertexOutput(_e21, _e23);
}
