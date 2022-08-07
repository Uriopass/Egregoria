struct Uniforms {
    u_view_proj: mat4x4<f32>,
}

struct VertexOutput {
    @location(0) out_color: vec4<f32>,
    @location(1) out_normal: vec3<f32>,
    @location(2) out_wpos: vec3<f32>,
    @location(3) out_uv: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

var<private> in_pos_1: vec3<f32>;
var<private> in_uv_1: vec2<f32>;
var<private> in_tint_1: vec4<f32>;
var<private> in_instance_pos_1: vec3<f32>;
var<private> in_dir_1: vec3<f32>;
var<private> in_scale_1: vec2<f32>;
var<private> out_color: vec4<f32>;
var<private> out_normal: vec3<f32>;
var<private> out_wpos: vec3<f32>;
var<private> out_uv: vec2<f32>;
@group(0) @binding(0) 
var<uniform> global: Uniforms;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    var x: vec3<f32>;
    var y: vec3<f32>;
    var z: vec3<f32>;
    var scaled: vec3<f32>;
    var wpos: vec3<f32>;

    let _e12: vec3<f32> = in_dir_1;
    x = _e12;
    _ = vec3<f32>(f32(0), f32(0), f32(1));
    _ = x;
    let _e29: vec3<f32> = x;
    y = cross(vec3<f32>(f32(0), f32(0), f32(1)), _e29);
    _ = x;
    _ = y;
    let _e34: vec3<f32> = y;
    _ = normalize(_e34);
    let _e36: vec3<f32> = x;
    _ = y;
    let _e38: vec3<f32> = y;
    z = cross(_e36, normalize(_e38));
    let _e42: vec3<f32> = in_pos_1;
    let _e44: vec2<f32> = in_scale_1;
    let _e45: vec2<f32> = (_e42.xy * _e44);
    let _e46: vec3<f32> = in_pos_1;
    scaled = vec3<f32>(_e45.x, _e45.y, _e46.z);
    let _e52: vec3<f32> = scaled;
    let _e54: vec3<f32> = x;
    let _e56: vec3<f32> = scaled;
    let _e58: vec3<f32> = y;
    let _e61: vec3<f32> = scaled;
    let _e63: vec3<f32> = z;
    let _e66: vec3<f32> = in_instance_pos_1;
    wpos = ((((_e52.x * _e54) + (_e56.y * _e58)) + (_e61.z * _e63)) + _e66);
    let _e70: mat4x4<f32> = global.u_view_proj;
    let _e71: vec3<f32> = wpos;
    gl_Position = (_e70 * vec4<f32>(_e71.x, _e71.y, _e71.z, 1.0));
    let _e78: vec4<f32> = in_tint_1;
    out_color = _e78;
    let _e79: vec3<f32> = z;
    out_normal = _e79;
    let _e80: vec3<f32> = wpos;
    out_wpos = _e80;
    let _e81: vec2<f32> = in_uv_1;
    out_uv = _e81;
    return;
}

@vertex 
fn main(@location(0) in_pos: vec3<f32>, @location(1) in_uv: vec2<f32>, @location(2) in_tint: vec4<f32>, @location(3) in_instance_pos: vec3<f32>, @location(4) in_dir: vec3<f32>, @location(5) in_scale: vec2<f32>) -> VertexOutput {
    in_pos_1 = in_pos;
    in_uv_1 = in_uv;
    in_tint_1 = in_tint;
    in_instance_pos_1 = in_instance_pos;
    in_dir_1 = in_dir;
    in_scale_1 = in_scale;
    main_1();
    let _e35: vec4<f32> = out_color;
    let _e37: vec3<f32> = out_normal;
    let _e39: vec3<f32> = out_wpos;
    let _e41: vec2<f32> = out_uv;
    let _e43: vec4<f32> = gl_Position;
    return VertexOutput(_e35, _e37, _e39, _e41, _e43);
}
