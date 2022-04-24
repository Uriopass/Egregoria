struct RenderParams {
    invproj: mat4x4<f32>;
    sunproj: mat4x4<f32>;
    cam_pos: vec4<f32>;
    sun: vec3<f32>;
    sun_col: vec4<f32>;
    viewport: vec2<f32>;
    time: f32;
    ssao_strength: f32;
    ssao_radius: f32;
    ssao_falloff: f32;
    ssao_base: f32;
    ssao_samples: i32;
    ssao_enabled: i32;
    shadow_mapping_enabled: i32;
    realistic_sky: i32;
};

struct Uni {
    params: RenderParams;
};

struct VertexOutput {
    [[location(0)]] out_pos: vec3<f32>;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> in_pos_1: vec3<f32>;
var<private> in_uv_1: vec2<f32>;
var<private> out_pos: vec3<f32>;
[[group(0), binding(0)]]
var<uniform> global: Uni;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    var near: vec4<f32>;
    var far: vec4<f32>;

    let _e6: vec3<f32> = in_pos_1;
    let _e7: vec2<f32> = _e6.xy;
    gl_Position = vec4<f32>(_e7.x, _e7.y, 0.9999998807907104, 1.0);
    let _e13: RenderParams = global.params;
    let _e15: vec3<f32> = in_pos_1;
    let _e16: vec2<f32> = _e15.xy;
    near = (_e13.invproj * vec4<f32>(_e16.x, _e16.y, -(1.0), 1.0));
    let _e25: RenderParams = global.params;
    let _e27: vec3<f32> = in_pos_1;
    let _e28: vec2<f32> = _e27.xy;
    far = (_e25.invproj * vec4<f32>(_e28.x, _e28.y, 1.0, 1.0));
    let _e36: vec4<f32> = far;
    let _e38: vec4<f32> = far;
    let _e42: vec4<f32> = near;
    let _e44: vec4<f32> = near;
    out_pos = ((_e36.xyz / vec3<f32>(_e38.w)) - (_e42.xyz / vec3<f32>(_e44.w)));
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] in_pos: vec3<f32>, [[location(1)]] in_uv: vec2<f32>) -> VertexOutput {
    in_pos_1 = in_pos;
    in_uv_1 = in_uv;
    main_1();
    let _e13: vec3<f32> = out_pos;
    let _e15: vec4<f32> = gl_Position;
    return VertexOutput(_e13, _e15);
}
