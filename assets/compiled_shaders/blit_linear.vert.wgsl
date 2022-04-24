struct VertexOutput {
    [[location(0)]] out_uv: vec2<f32>;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> in_pos_1: vec3<f32>;
var<private> in_uv_1: vec2<f32>;
var<private> out_uv: vec2<f32>;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    let _e4: vec3<f32> = in_pos_1;
    let _e5: vec2<f32> = _e4.xy;
    gl_Position = vec4<f32>(_e5.x, _e5.y, 1.0, 1.0);
    let _e11: vec2<f32> = in_uv_1;
    out_uv = _e11;
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] in_pos: vec3<f32>, [[location(1)]] in_uv: vec2<f32>) -> VertexOutput {
    in_pos_1 = in_pos;
    in_uv_1 = in_uv;
    main_1();
    let _e11: vec2<f32> = out_uv;
    let _e13: vec4<f32> = gl_Position;
    return VertexOutput(_e11, _e13);
}
