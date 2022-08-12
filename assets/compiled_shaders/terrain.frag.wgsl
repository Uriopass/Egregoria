#include "render_params.wgsl"

struct Uni {
    params: RenderParams,
}

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

var<private> in_normal_1: vec3<f32>;
var<private> in_wpos_1: vec3<f32>;
var<private> out_color: vec4<f32>;
@group(1) @binding(0) 
var<uniform> global: Uni;
@group(2) @binding(0) 
var t_terraindata: texture_2d<f32>;
@group(2) @binding(1) 
var s_terraindata: sampler;
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
    let _e14: vec4<f32> = gl_FragCoord;
    _ = (_e14.xy / vec2<f32>(512.0));
    let _e19: vec4<f32> = gl_FragCoord;
    let _e24: vec4<f32> = textureSample(t_bnoise, s_bnoise, (_e19.xy / vec2<f32>(512.0)));
    color = _e24.x;
    let _e27: f32 = color;
    return ((_e27 - 0.5) / 512.0);
}

fn sampleShadow() -> f32 {
    var light_local: vec4<f32>;
    var corrected: vec3<f32>;
    var total: f32 = 0.0;
    var offset: f32;
    var x: i32;
    var y: i32 = -1;

    let _e15: mat4x4<f32> = global.params.sunproj;
    let _e16: vec3<f32> = in_wpos_1;
    light_local = (_e15 * vec4<f32>(_e16.x, _e16.y, _e16.z, f32(1)));
    let _e25: vec4<f32> = light_local;
    let _e27: vec4<f32> = light_local;
    corrected = (((_e25.xyz / vec3<f32>(_e27.w)) * vec3<f32>(0.5, -(0.5), 1.0)) + vec3<f32>(0.5, 0.5, 0.0));
    let _e47: i32 = global.params.shadow_mapping_enabled;
    offset = (1.0 / f32(_e47));
    _ = -(1);
    loop {
        let _e55: i32 = y;
        if !((_e55 <= 1)) {
            break;
        }
        {
            x = -(1);
            loop {
                let _e64: i32 = x;
                if !((_e64 <= 1)) {
                    break;
                }
                {
                    let _e71: f32 = total;
                    let _e72: vec3<f32> = corrected;
                    let _e73: f32 = offset;
                    let _e74: i32 = x;
                    let _e75: i32 = y;
                    _ = (_e72 + (_e73 * vec3<f32>(f32(_e74), f32(_e75), -(1.0))));
                    let _e83: vec3<f32> = corrected;
                    let _e84: f32 = offset;
                    let _e85: i32 = x;
                    let _e86: i32 = y;
                    let _e93: vec3<f32> = (_e83 + (_e84 * vec3<f32>(f32(_e85), f32(_e86), -(1.0))));
                    let _e96: f32 = textureSampleCompare(t_sun_smap, s_sun_smap, _e93.xy, _e93.z);
                    total = (_e71 + _e96);
                }
                continuing {
                    let _e68: i32 = x;
                    x = (_e68 + 1);
                }
            }
        }
        continuing {
            let _e59: i32 = y;
            y = (_e59 + 1);
        }
    }
    let _e98: f32 = total;
    total = (_e98 / 9.0);
    let _e101: vec4<f32> = light_local;
    if (_e101.z >= 1.0) {
        {
            return 1.0;
        }
    }
    _ = total;
    let _e108: vec4<f32> = light_local;
    _ = _e108.xy;
    let _e110: vec4<f32> = light_local;
    _ = _e110.xy;
    let _e112: vec4<f32> = light_local;
    let _e114: vec4<f32> = light_local;
    _ = dot(_e112.xy, _e114.xy);
    let _e119: vec4<f32> = light_local;
    _ = _e119.xy;
    let _e121: vec4<f32> = light_local;
    _ = _e121.xy;
    let _e123: vec4<f32> = light_local;
    let _e125: vec4<f32> = light_local;
    _ = clamp(dot(_e123.xy, _e125.xy), 0.0, 1.0);
    let _e131: f32 = total;
    let _e134: vec4<f32> = light_local;
    _ = _e134.xy;
    let _e136: vec4<f32> = light_local;
    _ = _e136.xy;
    let _e138: vec4<f32> = light_local;
    let _e140: vec4<f32> = light_local;
    _ = dot(_e138.xy, _e140.xy);
    let _e145: vec4<f32> = light_local;
    _ = _e145.xy;
    let _e147: vec4<f32> = light_local;
    _ = _e147.xy;
    let _e149: vec4<f32> = light_local;
    let _e151: vec4<f32> = light_local;
    return mix(_e131, f32(1), clamp(dot(_e149.xy, _e151.xy), 0.0, 1.0));
}

fn grid() -> f32 {
    var level: f32;
    var w: f32 = 10000.0;
    var isIn: f32 = 0.0;
    var curgrid: vec2<f32>;
    var moved: vec2<f32>;
    var v: f32;
    var isOk: f32;

    _ = (&global.params);
    let _e14: vec3<f32> = in_wpos_1;
    _ = _e14.x;
    let _e16: vec3<f32> = in_wpos_1;
    let _e18: f32 = fwidth(_e16.x);
    level = (_e18 * f32(20));
    _ = f32(10000);
    let _e28: vec3<f32> = in_wpos_1;
    curgrid = (_e28.xy / vec2<f32>(f32(10000)));
    loop {
        let _e35: f32 = w;
        let _e36: f32 = level;
        if !((_e35 > (_e36 * f32(100)))) {
            break;
        }
        {
            let _e42: f32 = w;
            w = (_e42 / f32(5));
            let _e46: vec2<f32> = curgrid;
            curgrid = (_e46 * f32(5));
        }
    }
    loop {
        let _e50: f32 = w;
        let _e51: f32 = level;
        if !((_e50 > _e51)) {
            break;
        }
        {
            _ = curgrid;
            let _e55: vec2<f32> = curgrid;
            moved = fract(_e55);
            let _e58: vec2<f32> = moved;
            _ = _e58.x;
            let _e60: vec2<f32> = moved;
            _ = _e60.y;
            let _e62: vec2<f32> = moved;
            let _e64: vec2<f32> = moved;
            _ = min(_e62.x, _e64.y);
            let _e68: vec2<f32> = moved;
            _ = (f32(1) - _e68.x);
            let _e73: vec2<f32> = moved;
            _ = (f32(1) - _e73.y);
            let _e78: vec2<f32> = moved;
            let _e83: vec2<f32> = moved;
            _ = min((f32(1) - _e78.x), (f32(1) - _e83.y));
            let _e88: vec2<f32> = moved;
            _ = _e88.x;
            let _e90: vec2<f32> = moved;
            _ = _e90.y;
            let _e92: vec2<f32> = moved;
            let _e94: vec2<f32> = moved;
            let _e98: vec2<f32> = moved;
            _ = (f32(1) - _e98.x);
            let _e103: vec2<f32> = moved;
            _ = (f32(1) - _e103.y);
            let _e108: vec2<f32> = moved;
            let _e113: vec2<f32> = moved;
            v = min(min(_e92.x, _e94.y), min((f32(1) - _e108.x), (f32(1) - _e113.y)));
            _ = v;
            let _e126: f32 = v;
            let _e134: f32 = level;
            _ = ((_e134 * f32(100)) * 0.5);
            let _e140: f32 = level;
            _ = (_e140 * f32(100));
            _ = w;
            let _e145: f32 = level;
            let _e151: f32 = level;
            let _e155: f32 = w;
            isOk = (((f32(1) - smoothstep(0.004000000189989805, 0.004149999935179949, _e126)) * f32(2)) * (f32(1) - smoothstep(((_e145 * f32(100)) * 0.5), (_e151 * f32(100)), _e155)));
            _ = isIn;
            _ = isOk;
            let _e163: f32 = isIn;
            let _e164: f32 = isOk;
            isIn = max(_e163, _e164);
            let _e166: f32 = w;
            w = (_e166 / f32(5));
            let _e170: vec2<f32> = curgrid;
            curgrid = (_e170 * f32(5));
        }
    }
    let _e174: f32 = isIn;
    return _e174;
}

fn main_1() {
    var ssao: f32 = 1.0;
    var shadow_v: f32 = 1.0;
    var c: vec4<f32>;
    var normal: vec3<f32>;
    var cam: vec3<f32>;
    var L: vec3<f32>;
    var R: vec3<f32>;
    var V: vec3<f32>;
    var specular: f32;
    var sun_contrib: f32;
    var ambiant: vec3<f32>;
    var sun: f32;
    var final_rgb: vec3<f32>;

    _ = f32(1);
    let _e18: i32 = global.params.ssao_enabled;
    if (_e18 != 0) {
        {
            let _e21: vec4<f32> = gl_FragCoord;
            let _e24: vec2<f32> = global.params.viewport;
            _ = (_e21.xy / _e24);
            let _e26: vec4<f32> = gl_FragCoord;
            let _e29: vec2<f32> = global.params.viewport;
            let _e31: vec4<f32> = textureSample(t_ssao, s_ssao, (_e26.xy / _e29));
            ssao = _e31.x;
        }
    }
    _ = f32(1);
    let _e37: i32 = global.params.shadow_mapping_enabled;
    if (_e37 != 0) {
        {
            let _e40: f32 = sampleShadow();
            shadow_v = _e40;
        }
    }
    let _e42: vec4<f32> = global.params.grass_col;
    c = _e42;
    let _e45: i32 = global.params.grid_enabled;
    if (_e45 != 0) {
        {
            let _e49: vec4<f32> = c;
            let _e51: f32 = grid();
            c.y = (_e49.y + (_e51 * 0.014999999664723873));
        }
    }
    _ = global.params.sand_col;
    _ = c;
    _ = -(5.0);
    let _e61: vec3<f32> = in_wpos_1;
    _ = _e61.z;
    let _e66: vec3<f32> = in_wpos_1;
    _ = smoothstep(-(5.0), 0.0, _e66.z);
    let _e70: vec4<f32> = global.params.sand_col;
    let _e71: vec4<f32> = c;
    _ = -(5.0);
    let _e75: vec3<f32> = in_wpos_1;
    _ = _e75.z;
    let _e80: vec3<f32> = in_wpos_1;
    c = mix(_e70, _e71, vec4<f32>(smoothstep(-(5.0), 0.0, _e80.z)));
    _ = global.params.sea_col;
    _ = c;
    _ = -(25.0);
    _ = -(20.0);
    let _e92: vec3<f32> = in_wpos_1;
    _ = _e92.z;
    let _e98: vec3<f32> = in_wpos_1;
    _ = smoothstep(-(25.0), -(20.0), _e98.z);
    let _e102: vec4<f32> = global.params.sea_col;
    let _e103: vec4<f32> = c;
    _ = -(25.0);
    _ = -(20.0);
    let _e108: vec3<f32> = in_wpos_1;
    _ = _e108.z;
    let _e114: vec3<f32> = in_wpos_1;
    c = mix(_e102, _e103, vec4<f32>(smoothstep(-(25.0), -(20.0), _e114.z)));
    _ = in_normal_1;
    let _e120: vec3<f32> = in_normal_1;
    normal = normalize(_e120);
    let _e124: vec4<f32> = global.params.cam_pos;
    cam = _e124.xyz;
    let _e128: vec3<f32> = global.params.sun;
    L = _e128;
    let _e131: vec3<f32> = normal;
    _ = normal;
    _ = L;
    let _e136: vec3<f32> = normal;
    let _e137: vec3<f32> = L;
    let _e140: vec3<f32> = L;
    _ = (((f32(2) * _e131) * dot(_e136, _e137)) - _e140);
    let _e143: vec3<f32> = normal;
    _ = normal;
    _ = L;
    let _e148: vec3<f32> = normal;
    let _e149: vec3<f32> = L;
    let _e152: vec3<f32> = L;
    R = normalize((((f32(2) * _e143) * dot(_e148, _e149)) - _e152));
    let _e156: vec3<f32> = cam;
    let _e157: vec3<f32> = in_wpos_1;
    _ = (_e156 - _e157);
    let _e159: vec3<f32> = cam;
    let _e160: vec3<f32> = in_wpos_1;
    V = normalize((_e159 - _e160));
    _ = R;
    _ = V;
    let _e166: vec3<f32> = R;
    let _e167: vec3<f32> = V;
    _ = dot(_e166, _e167);
    _ = R;
    _ = V;
    let _e173: vec3<f32> = R;
    let _e174: vec3<f32> = V;
    specular = clamp(dot(_e173, _e174), 0.0, 1.0);
    _ = specular;
    let _e182: f32 = specular;
    specular = pow(_e182, f32(2));
    _ = normal;
    _ = global.params.sun;
    let _e189: vec3<f32> = normal;
    let _e191: vec3<f32> = global.params.sun;
    _ = dot(_e189, _e191);
    _ = normal;
    _ = global.params.sun;
    let _e198: vec3<f32> = normal;
    let _e200: vec3<f32> = global.params.sun;
    sun_contrib = clamp(dot(_e198, _e200), 0.0, 1.0);
    let _e207: vec4<f32> = c;
    ambiant = (0.15000000596046448 * _e207.xyz);
    let _e212: f32 = sun_contrib;
    let _e215: f32 = specular;
    let _e218: f32 = shadow_v;
    sun = (((0.8500000238418579 * _e212) + (0.5 * _e215)) * _e218);
    let _e221: vec3<f32> = ambiant;
    final_rgb = _e221;
    let _e223: vec3<f32> = final_rgb;
    let _e224: f32 = sun;
    let _e226: vec4<f32> = global.params.sun_col;
    let _e228: vec4<f32> = c;
    final_rgb = (_e223 + (_e224 * (_e226.xyz * _e228.xyz)));
    let _e233: vec3<f32> = final_rgb;
    let _e234: f32 = ssao;
    final_rgb = (_e233 * _e234);
    let _e236: vec3<f32> = final_rgb;
    let _e237: f32 = dither();
    final_rgb = (_e236 + vec3<f32>(_e237));
    let _e240: vec3<f32> = final_rgb;
    let _e241: vec4<f32> = c;
    out_color = vec4<f32>(_e240.x, _e240.y, _e240.z, _e241.w);
    return;
}

@fragment 
fn main(@location(0) in_normal: vec3<f32>, @location(1) in_wpos: vec3<f32>, @builtin(position) param: vec4<f32>) -> FragmentOutput {
    in_normal_1 = in_normal;
    in_wpos_1 = in_wpos;
    gl_FragCoord = param;
    _ = (&global.params);
    main_1();
    let _e31: vec4<f32> = out_color;
    return FragmentOutput(_e31);
}
