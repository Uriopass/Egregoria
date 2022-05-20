struct RenderParams {
    invproj: mat4x4<f32>;
    sunproj: mat4x4<f32>;
    cam_dir: vec4<f32>;
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

var<private> in_pos_1: vec3<f32>;
var<private> in_normal_1: vec3<f32>;
var<private> in_uv_1: vec2<f32>;
var<private> in_color_1: vec4<f32>;
var<private> in_instance_pos_1: vec3<f32>;
var<private> in_instance_dir_1: vec3<f32>;
var<private> in_instance_tint_1: vec4<f32>;
var<private> out_color: vec4<f32>;
var<private> out_normal: vec3<f32>;
var<private> out_wpos: vec3<f32>;
var<private> out_uv: vec2<f32>;
[[group(0), binding(0)]]
var<uniform> global: Uniforms;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    var x: vec3<f32>;
    var y: vec3<f32>;
    var z: vec3<f32>;
    var off: vec3<f32>;
    var normal: vec3<f32>;

    let _e13: vec3<f32> = in_instance_dir_1;
    x = _e13;
    let _e30: vec3<f32> = x;
    y = cross(vec3<f32>(f32(0), f32(0), f32(1)), _e30);
    let _e35: vec3<f32> = y;
    let _e37: vec3<f32> = x;
    let _e39: vec3<f32> = y;
    z = cross(_e37, normalize(_e39));
    let _e43: vec3<f32> = in_pos_1;
    let _e45: vec3<f32> = x;
    let _e47: vec3<f32> = in_pos_1;
    let _e49: vec3<f32> = y;
    let _e52: vec3<f32> = in_pos_1;
    let _e54: vec3<f32> = z;
    let _e57: vec3<f32> = in_instance_pos_1;
    off = ((((_e43.x * _e45) + (_e47.y * _e49)) + (_e52.z * _e54)) + _e57);
    let _e60: vec3<f32> = in_normal_1;
    let _e62: vec3<f32> = x;
    let _e64: vec3<f32> = in_normal_1;
    let _e66: vec3<f32> = y;
    let _e69: vec3<f32> = in_normal_1;
    let _e71: vec3<f32> = z;
    normal = (((_e60.x * _e62) + (_e64.y * _e66)) + (_e69.z * _e71));
    let _e76: mat4x4<f32> = global.u_view_proj;
    let _e77: vec3<f32> = off;
    gl_Position = (_e76 * vec4<f32>(_e77.x, _e77.y, _e77.z, 1.0));
    let _e84: vec4<f32> = in_instance_tint_1;
    let _e85: vec4<f32> = in_color_1;
    out_color = (_e84 * _e85);
    let _e87: vec3<f32> = normal;
    out_normal = _e87;
    let _e88: vec3<f32> = off;
    out_wpos = _e88;
    let _e89: vec2<f32> = in_uv_1;
    out_uv = _e89;
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] in_pos: vec3<f32>, [[location(1)]] in_normal: vec3<f32>, [[location(2)]] in_uv: vec2<f32>, [[location(3)]] in_color: vec4<f32>, [[location(4)]] in_instance_pos: vec3<f32>, [[location(5)]] in_instance_dir: vec3<f32>, [[location(6)]] in_instance_tint: vec4<f32>) -> VertexOutput {
    in_pos_1 = in_pos;
    in_normal_1 = in_normal;
    in_uv_1 = in_uv;
    in_color_1 = in_color;
    in_instance_pos_1 = in_instance_pos;
    in_instance_dir_1 = in_instance_dir;
    in_instance_tint_1 = in_instance_tint;
    main_1();
    let _e39: vec4<f32> = out_color;
    let _e41: vec3<f32> = out_normal;
    let _e43: vec3<f32> = out_wpos;
    let _e45: vec2<f32> = out_uv;
    let _e47: vec4<f32> = gl_Position;
    return VertexOutput(_e39, _e41, _e43, _e45, _e47);
}
