#include "render_params.wgsl"

struct Uni {
    params: RenderParams,
}

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

var<private> in_tint_1: vec4<f32>;
var<private> in_normal_1: vec3<f32>;
var<private> in_wpos_1: vec3<f32>;
var<private> in_uv_1: vec2<f32>;
var<private> out_color: vec4<f32>;
@group(1) @binding(0) 
var<uniform> global: Uni;
@group(2) @binding(0) 
var t_albedo: texture_2d<f32>;
@group(2) @binding(1) 
var s_albedo: sampler;
@group(3) @binding(0) 
var t_ssao: texture_2d<f32>;
@group(3) @binding(1) 
var s_ssao: sampler;
@group(3) @binding(2) 
var t_bnoise: texture_2d<f32>;
@group(3) @binding(3) 
var s_bnoise: sampler;
@group(3) @binding(4) 
var t_sun_smap: texture_depth_2d;
@group(3) @binding(5) 
var s_sun_smap: sampler_comparison;
var<private> gl_FragCoord: vec4<f32>;

fn dither() -> f32 {
    var color: f32;

    _ = (&global.params);
    let _e16: vec4<f32> = gl_FragCoord;
    _ = (_e16.xy / vec2<f32>(512.0));
    let _e21: vec4<f32> = gl_FragCoord;
    let _e26: vec4<f32> = textureSample(t_bnoise, s_bnoise, (_e21.xy / vec2<f32>(512.0)));
    color = _e26.x;
    let _e29: f32 = color;
    return ((_e29 - 0.5) / 512.0);
}

fn sampleShadow() -> f32 {
    var light_local: vec4<f32>;
    var corrected: vec3<f32>;
    var total: f32 = 0.0;
    var offset: f32;
    var x: i32;
    var y: i32 = -1;

    let _e17: mat4x4<f32> = global.params.sunproj;
    let _e18: vec3<f32> = in_wpos_1;
    light_local = (_e17 * vec4<f32>(_e18.x, _e18.y, _e18.z, f32(1)));
    let _e27: vec4<f32> = light_local;
    let _e29: vec4<f32> = light_local;
    corrected = (((_e27.xyz / vec3<f32>(_e29.w)) * vec3<f32>(0.5, -(0.5), 1.0)) + vec3<f32>(0.5, 0.5, 0.0));
    let _e49: i32 = global.params.shadow_mapping_enabled;
    offset = (1.0 / f32(_e49));
    _ = -(1);
    loop {
        let _e57: i32 = y;
        if !((_e57 <= 1)) {
            break;
        }
        {
            x = -(1);
            loop {
                let _e66: i32 = x;
                if !((_e66 <= 1)) {
                    break;
                }
                {
                    let _e73: f32 = total;
                    let _e74: vec3<f32> = corrected;
                    let _e75: f32 = offset;
                    let _e76: i32 = x;
                    let _e77: i32 = y;
                    _ = (_e74 + (_e75 * vec3<f32>(f32(_e76), f32(_e77), -(1.0))));
                    let _e85: vec3<f32> = corrected;
                    let _e86: f32 = offset;
                    let _e87: i32 = x;
                    let _e88: i32 = y;
                    let _e95: vec3<f32> = (_e85 + (_e86 * vec3<f32>(f32(_e87), f32(_e88), -(1.0))));
                    let _e98: f32 = textureSampleCompare(t_sun_smap, s_sun_smap, _e95.xy, _e95.z);
                    total = (_e73 + _e98);
                }
                continuing {
                    let _e70: i32 = x;
                    x = (_e70 + 1);
                }
            }
        }
        continuing {
            let _e61: i32 = y;
            y = (_e61 + 1);
        }
    }
    let _e100: f32 = total;
    total = (_e100 / 9.0);
    let _e103: vec4<f32> = light_local;
    if (_e103.z >= 1.0) {
        {
            return 1.0;
        }
    }
    _ = total;
    let _e110: vec4<f32> = light_local;
    _ = _e110.xy;
    let _e112: vec4<f32> = light_local;
    _ = _e112.xy;
    let _e114: vec4<f32> = light_local;
    let _e116: vec4<f32> = light_local;
    _ = dot(_e114.xy, _e116.xy);
    let _e121: vec4<f32> = light_local;
    _ = _e121.xy;
    let _e123: vec4<f32> = light_local;
    _ = _e123.xy;
    let _e125: vec4<f32> = light_local;
    let _e127: vec4<f32> = light_local;
    _ = clamp(dot(_e125.xy, _e127.xy), 0.0, 1.0);
    let _e133: f32 = total;
    let _e136: vec4<f32> = light_local;
    _ = _e136.xy;
    let _e138: vec4<f32> = light_local;
    _ = _e138.xy;
    let _e140: vec4<f32> = light_local;
    let _e142: vec4<f32> = light_local;
    _ = dot(_e140.xy, _e142.xy);
    let _e147: vec4<f32> = light_local;
    _ = _e147.xy;
    let _e149: vec4<f32> = light_local;
    _ = _e149.xy;
    let _e151: vec4<f32> = light_local;
    let _e153: vec4<f32> = light_local;
    return mix(_e133, f32(1), clamp(dot(_e151.xy, _e153.xy), 0.0, 1.0));
}

fn main_1() {
    var albedo: vec4<f32>;
    var ssao: f32 = 1.0;
    var shadow_v: f32 = 1.0;
    var normal: vec3<f32>;
    var cam: vec3<f32>;
    var L: vec3<f32>;
    var R: vec3<f32>;
    var V: vec3<f32>;
    var specular: f32;
    var sun_contrib: f32;
    var c: vec4<f32>;
    var ambiant: vec3<f32>;
    var sun: f32;
    var final_rgb: vec3<f32>;

    _ = in_uv_1;
    let _e17: vec2<f32> = in_uv_1;
    let _e18: vec4<f32> = textureSample(t_albedo, s_albedo, _e17);
    albedo = _e18;
    _ = f32(1);
    let _e24: i32 = global.params.ssao_enabled;
    if (_e24 != 0) {
        {
            let _e27: vec4<f32> = gl_FragCoord;
            let _e30: vec2<f32> = global.params.viewport;
            _ = (_e27.xy / _e30);
            let _e32: vec4<f32> = gl_FragCoord;
            let _e35: vec2<f32> = global.params.viewport;
            let _e37: vec4<f32> = textureSample(t_ssao, s_ssao, (_e32.xy / _e35));
            ssao = _e37.x;
        }
    }
    _ = f32(1);
    let _e43: i32 = global.params.shadow_mapping_enabled;
    if (_e43 != 0) {
        {
            let _e46: f32 = sampleShadow();
            shadow_v = _e46;
        }
    }
    _ = in_normal_1;
    let _e48: vec3<f32> = in_normal_1;
    normal = normalize(_e48);
    let _e52: vec4<f32> = global.params.cam_pos;
    cam = _e52.xyz;
    let _e56: vec3<f32> = global.params.sun;
    L = _e56;
    let _e59: vec3<f32> = normal;
    _ = normal;
    _ = L;
    let _e64: vec3<f32> = normal;
    let _e65: vec3<f32> = L;
    let _e68: vec3<f32> = L;
    _ = (((f32(2) * _e59) * dot(_e64, _e65)) - _e68);
    let _e71: vec3<f32> = normal;
    _ = normal;
    _ = L;
    let _e76: vec3<f32> = normal;
    let _e77: vec3<f32> = L;
    let _e80: vec3<f32> = L;
    R = normalize((((f32(2) * _e71) * dot(_e76, _e77)) - _e80));
    let _e84: vec3<f32> = cam;
    let _e85: vec3<f32> = in_wpos_1;
    _ = (_e84 - _e85);
    let _e87: vec3<f32> = cam;
    let _e88: vec3<f32> = in_wpos_1;
    V = normalize((_e87 - _e88));
    _ = R;
    _ = V;
    let _e94: vec3<f32> = R;
    let _e95: vec3<f32> = V;
    _ = dot(_e94, _e95);
    _ = R;
    _ = V;
    let _e101: vec3<f32> = R;
    let _e102: vec3<f32> = V;
    specular = clamp(dot(_e101, _e102), 0.0, 1.0);
    _ = specular;
    let _e110: f32 = specular;
    specular = pow(_e110, f32(5));
    _ = normal;
    _ = global.params.sun;
    let _e117: vec3<f32> = normal;
    let _e119: vec3<f32> = global.params.sun;
    _ = dot(_e117, _e119);
    _ = normal;
    _ = global.params.sun;
    let _e126: vec3<f32> = normal;
    let _e128: vec3<f32> = global.params.sun;
    sun_contrib = clamp(dot(_e126, _e128), 0.0, 1.0);
    let _e134: vec4<f32> = in_tint_1;
    let _e135: vec4<f32> = albedo;
    c = (_e134 * _e135);
    let _e139: vec4<f32> = c;
    ambiant = (0.15000000596046448 * _e139.xyz);
    let _e144: f32 = sun_contrib;
    let _e147: f32 = specular;
    let _e150: f32 = shadow_v;
    sun = (((0.8500000238418579 * _e144) + (0.5 * _e147)) * _e150);
    let _e153: vec3<f32> = ambiant;
    final_rgb = _e153;
    let _e155: vec3<f32> = final_rgb;
    let _e156: f32 = sun;
    let _e158: vec4<f32> = global.params.sun_col;
    let _e160: vec4<f32> = c;
    final_rgb = (_e155 + (_e156 * (_e158.xyz * _e160.xyz)));
    let _e165: vec3<f32> = final_rgb;
    let _e166: f32 = ssao;
    final_rgb = (_e165 * _e166);
    let _e168: vec3<f32> = final_rgb;
    let _e169: f32 = dither();
    final_rgb = (_e168 + vec3<f32>(_e169));
    let _e172: vec3<f32> = final_rgb;
    let _e173: vec4<f32> = c;
    out_color = vec4<f32>(_e172.x, _e172.y, _e172.z, _e173.w);
    return;
}

@fragment 
fn main(@location(0) in_tint: vec4<f32>, @location(1) in_normal: vec3<f32>, @location(2) in_wpos: vec3<f32>, @location(3) in_uv: vec2<f32>, @builtin(position) param: vec4<f32>) -> FragmentOutput {
    in_tint_1 = in_tint;
    in_normal_1 = in_normal;
    in_wpos_1 = in_wpos;
    in_uv_1 = in_uv;
    gl_FragCoord = param;
    _ = (&global.params);
    main_1();
    let _e39: vec4<f32> = out_color;
    return FragmentOutput(_e39);
}
