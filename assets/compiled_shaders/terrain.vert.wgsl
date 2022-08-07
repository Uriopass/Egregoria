struct Uniforms {
    u_view_proj: mat4x4<f32>,
}

struct VertexOutput {
    @location(0) out_normal: vec3<f32>,
    @location(1) out_wpos: vec3<f32>,
    @builtin(position) member: vec4<f32>,
}

var<private> in_position_1: vec2<f32>;
var<private> in_off_1: vec2<f32>;
var<private> out_normal: vec3<f32>;
var<private> out_wpos: vec3<f32>;
@group(0) @binding(0) 
var<uniform> global: Uniforms;
@group(2) @binding(0) 
var t_terraindata: texture_2d<f32>;
@group(2) @binding(1) 
var s_terraindata: sampler;
var<private> gl_Position: vec4<f32>;

fn main_1() {
    var tpos: vec2<i32>;
    var height: f32;
    var hx: f32;
    var hy: f32;
    var pos: vec3<f32>;

    let _e8: vec2<f32> = in_position_1;
    let _e9: vec2<f32> = in_off_1;
    tpos = vec2<i32>(((_e8 + _e9) / vec2<f32>(f32(32))));
    _ = tpos;
    let _e19: vec2<i32> = tpos;
    let _e21: vec4<f32> = textureLoad(t_terraindata, _e19, 0);
    height = _e21.x;
    let _e27: vec2<i32> = tpos;
    _ = (vec2<i32>(1, 0) + _e27);
    let _e33: vec2<i32> = tpos;
    let _e36: vec4<f32> = textureLoad(t_terraindata, (vec2<i32>(1, 0) + _e33), 0);
    hx = _e36.x;
    let _e42: vec2<i32> = tpos;
    _ = (vec2<i32>(0, 1) + _e42);
    let _e48: vec2<i32> = tpos;
    let _e51: vec4<f32> = textureLoad(t_terraindata, (vec2<i32>(0, 1) + _e48), 0);
    hy = _e51.x;
    let _e54: vec2<f32> = in_position_1;
    let _e55: vec2<f32> = in_off_1;
    let _e56: vec2<f32> = (_e54 + _e55);
    let _e57: f32 = height;
    pos = vec3<f32>(_e56.x, _e56.y, _e57);
    let _e62: vec3<f32> = pos;
    out_wpos = _e62;
    let _e65: f32 = hx;
    let _e66: f32 = height;
    _ = vec3<f32>(f32(32), f32(0), (_e65 - _e66));
    let _e73: f32 = hy;
    let _e74: f32 = height;
    _ = vec3<f32>(f32(0), f32(32), (_e73 - _e74));
    let _e81: f32 = hx;
    let _e82: f32 = height;
    let _e89: f32 = hy;
    let _e90: f32 = height;
    _ = cross(vec3<f32>(f32(32), f32(0), (_e81 - _e82)), vec3<f32>(f32(0), f32(32), (_e89 - _e90)));
    let _e98: f32 = hx;
    let _e99: f32 = height;
    _ = vec3<f32>(f32(32), f32(0), (_e98 - _e99));
    let _e106: f32 = hy;
    let _e107: f32 = height;
    _ = vec3<f32>(f32(0), f32(32), (_e106 - _e107));
    let _e114: f32 = hx;
    let _e115: f32 = height;
    let _e122: f32 = hy;
    let _e123: f32 = height;
    out_normal = normalize(cross(vec3<f32>(f32(32), f32(0), (_e114 - _e115)), vec3<f32>(f32(0), f32(32), (_e122 - _e123))));
    let _e131: mat4x4<f32> = global.u_view_proj;
    let _e132: vec3<f32> = pos;
    gl_Position = (_e131 * vec4<f32>(_e132.x, _e132.y, _e132.z, 1.0));
    return;
}

@vertex 
fn main(@location(0) in_position: vec2<f32>, @location(1) in_off: vec2<f32>) -> VertexOutput {
    in_position_1 = in_position;
    in_off_1 = in_off;
    _ = (&global.u_view_proj);
    main_1();
    let _e19: vec3<f32> = out_normal;
    let _e21: vec3<f32> = out_wpos;
    let _e23: vec4<f32> = gl_Position;
    return VertexOutput(_e19, _e21, _e23);
}
