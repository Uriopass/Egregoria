struct RenderParams {
    invproj: mat4x4<f32>,
    sunproj: mat4x4<f32>,
    cam_pos: vec4<f32>,
    cam_dir: vec4<f32>,
    sun: vec3<f32>,
    sun_col: vec4<f32>,
    grass_col: vec4<f32>,
    sand_col: vec4<f32>,
    sea_col: vec4<f32>,
    viewport: vec2<f32>,
    time: f32,
    ssao_strength: f32,
    ssao_radius: f32,
    ssao_falloff: f32,
    ssao_base: f32,
    ssao_samples: i32,
    ssao_enabled: i32,
    shadow_mapping_enabled: i32,
    realistic_sky: i32,
    grid_enabled: i32,
}

struct Uni {
    params: RenderParams,
}

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

var<private> in_pos_1: vec3<f32>;
var<private> out_color: vec4<f32>;
@group(0) @binding(0) 
var<uniform> global: Uni;
@group(1) @binding(0) 
var t_bnoise: texture_2d<f32>;
@group(1) @binding(1) 
var s_bnoise: sampler;
@group(2) @binding(0) 
var t_gradientsky: texture_2d<f32>;
@group(2) @binding(1) 
var s_gradientsky: sampler;
@group(2) @binding(2) 
var t_starfield: texture_2d<f32>;
@group(2) @binding(3) 
var s_starfield: sampler;
var<private> gl_FragCoord: vec4<f32>;

fn rsi(r0_: vec3<f32>, rd: vec3<f32>, sr: f32) -> vec2<f32> {
    var r0_1: vec3<f32>;
    var rd_1: vec3<f32>;
    var sr_1: f32;
    var a: f32;
    var b: f32;
    var c: f32;
    var d: f32;

    _ = (&global.params);
    r0_1 = r0_;
    rd_1 = rd;
    sr_1 = sr;
    _ = rd_1;
    _ = rd_1;
    let _e18: vec3<f32> = rd_1;
    let _e19: vec3<f32> = rd_1;
    a = dot(_e18, _e19);
    _ = rd_1;
    _ = r0_1;
    let _e25: vec3<f32> = rd_1;
    let _e26: vec3<f32> = r0_1;
    b = (2.0 * dot(_e25, _e26));
    _ = r0_1;
    _ = r0_1;
    let _e32: vec3<f32> = r0_1;
    let _e33: vec3<f32> = r0_1;
    let _e35: f32 = sr_1;
    let _e36: f32 = sr_1;
    c = (dot(_e32, _e33) - (_e35 * _e36));
    let _e40: f32 = b;
    let _e41: f32 = b;
    let _e44: f32 = a;
    let _e46: f32 = c;
    d = ((_e40 * _e41) - ((4.0 * _e44) * _e46));
    let _e50: f32 = d;
    if (_e50 < 0.0) {
        return vec2<f32>(100000.0, -(100000.0));
    }
    let _e57: f32 = b;
    _ = d;
    let _e60: f32 = d;
    let _e64: f32 = a;
    let _e67: f32 = b;
    _ = d;
    let _e70: f32 = d;
    let _e74: f32 = a;
    return vec2<f32>(((-(_e57) - sqrt(_e60)) / (2.0 * _e64)), ((-(_e67) + sqrt(_e70)) / (2.0 * _e74)));
}

fn atmosphere(r: vec3<f32>, pSun: vec3<f32>) -> vec3<f32> {
    var r_1: vec3<f32>;
    var pSun_1: vec3<f32>;
    var p: vec2<f32>;
    var iStepSize: f32;
    var iTime: f32;
    var totalRlh: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    var totalMie: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    var iOdRlh: f32 = 0.0;
    var iOdMie: f32 = 0.0;
    var mu: f32;
    var mumu: f32;
    var gg: f32 = 0.5745640245780947;
    var pRlh: f32;
    var pMie: f32;
    var i: i32 = 0;
    var iPos: vec3<f32>;
    var iHeight: f32;
    var odStepRlh: f32;
    var odStepMie: f32;
    var jStepSize: f32;
    var jTime: f32 = 0.0;
    var jOdRlh: f32 = 0.0;
    var jOdMie: f32 = 0.0;
    var j: i32 = 0;
    var jPos: vec3<f32>;
    var jHeight: f32;
    var attn: vec3<f32>;

    _ = (&global.params);
    r_1 = r;
    pSun_1 = pSun;
    _ = r_1;
    let _e24: vec3<f32> = r_1;
    let _e25: vec2<f32> = rsi(vec3<f32>(0.0, 6372000.0, 0.0), _e24, 6471000.0);
    p = _e25;
    let _e27: vec2<f32> = p;
    let _e29: vec2<f32> = p;
    if (_e27.x > _e29.y) {
        return vec3<f32>(f32(0), f32(0), f32(0));
    }
    let _e40: vec2<f32> = p;
    _ = _e40.y;
    _ = r_1;
    let _e43: vec3<f32> = r_1;
    let _e44: vec2<f32> = rsi(vec3<f32>(0.0, 6372000.0, 0.0), _e43, 6371000.0);
    _ = _e44.x;
    let _e46: vec2<f32> = p;
    _ = r_1;
    let _e49: vec3<f32> = r_1;
    let _e50: vec2<f32> = rsi(vec3<f32>(0.0, 6372000.0, 0.0), _e49, 6371000.0);
    p.y = min(_e46.y, _e50.x);
    let _e53: vec2<f32> = p;
    let _e55: vec2<f32> = p;
    iStepSize = ((_e53.y - _e55.x) / f32(12));
    let _e62: f32 = iStepSize;
    iTime = (_e62 * 0.375);
    _ = vec3<f32>(f32(0), f32(0), f32(0));
    _ = vec3<f32>(f32(0), f32(0), f32(0));
    _ = r_1;
    _ = pSun_1;
    let _e88: vec3<f32> = r_1;
    let _e89: vec3<f32> = pSun_1;
    mu = dot(_e88, _e89);
    let _e92: f32 = mu;
    let _e93: f32 = mu;
    mumu = (_e92 * _e93);
    _ = (0.7580000162124634 * 0.7580000162124634);
    let _e104: f32 = mumu;
    pRlh = ((3.0 / (16.0 * 3.141592025756836)) * (1.0 + _e104));
    let _e114: f32 = gg;
    let _e116: f32 = mumu;
    let _e122: f32 = gg;
    let _e125: f32 = mu;
    _ = ((1.0 + _e122) - ((2.0 * _e125) * 0.7580000162124634));
    let _e131: f32 = gg;
    let _e134: f32 = mu;
    let _e141: f32 = gg;
    pMie = (((3.0 / (8.0 * 3.141592025756836)) * ((1.0 - _e114) * (_e116 + 1.0))) / (pow(((1.0 + _e131) - ((2.0 * _e134) * 0.7580000162124634)), 1.5) * (2.0 + _e141)));
    loop {
        let _e148: i32 = i;
        if !((_e148 < 12)) {
            break;
        }
        {
            let _e155: vec3<f32> = r_1;
            let _e156: f32 = iTime;
            iPos = (vec3<f32>(0.0, 6372000.0, 0.0) + (_e155 * _e156));
            _ = iPos;
            let _e161: vec3<f32> = iPos;
            iHeight = (length(_e161) - 6371000.0);
            let _e165: f32 = iHeight;
            _ = (-(_e165) / 8000.0);
            let _e168: f32 = iHeight;
            let _e172: f32 = iStepSize;
            odStepRlh = (exp((-(_e168) / 8000.0)) * _e172);
            let _e175: f32 = iHeight;
            _ = (-(_e175) / 1200.0);
            let _e178: f32 = iHeight;
            let _e182: f32 = iStepSize;
            odStepMie = (exp((-(_e178) / 1200.0)) * _e182);
            let _e185: f32 = iOdRlh;
            let _e186: f32 = odStepRlh;
            iOdRlh = (_e185 + _e186);
            let _e188: f32 = iOdMie;
            let _e189: f32 = odStepMie;
            iOdMie = (_e188 + _e189);
            _ = iPos;
            _ = pSun_1;
            let _e193: vec3<f32> = iPos;
            let _e194: vec3<f32> = pSun_1;
            let _e195: vec2<f32> = rsi(_e193, _e194, 6471000.0);
            jStepSize = (_e195.y / f32(4));
            loop {
                let _e209: i32 = j;
                if !((_e209 < 4)) {
                    break;
                }
                {
                    let _e216: vec3<f32> = iPos;
                    let _e217: vec3<f32> = pSun_1;
                    let _e218: f32 = jTime;
                    let _e219: f32 = jStepSize;
                    jPos = (_e216 + (_e217 * (_e218 + (_e219 * 0.5))));
                    _ = jPos;
                    let _e227: vec3<f32> = jPos;
                    jHeight = (length(_e227) - 6371000.0);
                    let _e231: f32 = jOdRlh;
                    let _e232: f32 = jHeight;
                    _ = (-(_e232) / 8000.0);
                    let _e235: f32 = jHeight;
                    let _e239: f32 = jStepSize;
                    jOdRlh = (_e231 + (exp((-(_e235) / 8000.0)) * _e239));
                    let _e242: f32 = jOdMie;
                    let _e243: f32 = jHeight;
                    _ = (-(_e243) / 1200.0);
                    let _e246: f32 = jHeight;
                    let _e250: f32 = jStepSize;
                    jOdMie = (_e242 + (exp((-(_e246) / 1200.0)) * _e250));
                    let _e253: f32 = jTime;
                    let _e254: f32 = jStepSize;
                    jTime = (_e253 + _e254);
                }
                continuing {
                    let _e213: i32 = j;
                    j = (_e213 + 1);
                }
            }
            let _e256: f32 = iOdMie;
            let _e257: f32 = jOdMie;
            let _e260: f32 = iOdRlh;
            let _e261: f32 = jOdRlh;
            _ = -((vec3<f32>((2.099999983329326e-5 * (_e256 + _e257))) + (vec3<f32>(5.500000042957254e-6, 1.2999999853491317e-5, 2.2399999579647556e-5) * (_e260 + _e261))));
            let _e267: f32 = iOdMie;
            let _e268: f32 = jOdMie;
            let _e271: f32 = iOdRlh;
            let _e272: f32 = jOdRlh;
            attn = exp(-((vec3<f32>((2.099999983329326e-5 * (_e267 + _e268))) + (vec3<f32>(5.500000042957254e-6, 1.2999999853491317e-5, 2.2399999579647556e-5) * (_e271 + _e272)))));
            let _e280: vec3<f32> = totalRlh;
            let _e281: f32 = odStepRlh;
            let _e282: vec3<f32> = attn;
            totalRlh = (_e280 + (_e281 * _e282));
            let _e285: vec3<f32> = totalMie;
            let _e286: f32 = odStepMie;
            let _e287: vec3<f32> = attn;
            totalMie = (_e285 + (_e286 * _e287));
            let _e290: f32 = iTime;
            let _e291: f32 = iStepSize;
            iTime = (_e290 + _e291);
        }
        continuing {
            let _e152: i32 = i;
            i = (_e152 + 1);
        }
    }
    let _e293: f32 = pRlh;
    let _e295: vec3<f32> = totalRlh;
    let _e297: f32 = pMie;
    let _e299: vec3<f32> = totalMie;
    return (22.0 * (((_e293 * vec3<f32>(5.500000042957254e-6, 1.2999999853491317e-5, 2.2399999579647556e-5)) * _e295) + ((_e297 * 2.099999983329326e-5) * _e299)));
}

fn dither() -> f32 {
    var color: f32;

    _ = (&global.params);
    let _e20: vec4<f32> = gl_FragCoord;
    _ = (_e20.xy / vec2<f32>(512.0));
    let _e25: vec4<f32> = gl_FragCoord;
    let _e30: vec4<f32> = textureSample(t_bnoise, s_bnoise, (_e25.xy / vec2<f32>(512.0)));
    color = _e30.x;
    let _e33: f32 = color;
    return ((_e33 - 0.5) / 255.0);
}

fn atan2_(y: f32, x: f32) -> f32 {
    var y_1: f32;
    var x_1: f32;
    var s: bool;

    _ = (&global.params);
    y_1 = y;
    x_1 = x;
    _ = x_1;
    let _e25: f32 = x_1;
    _ = y_1;
    let _e28: f32 = y_1;
    s = (abs(_e25) > abs(_e28));
    _ = x_1;
    _ = y_1;
    let _e37: f32 = x_1;
    let _e38: f32 = y_1;
    _ = ((3.141592025756836 / 2.0) - atan2(_e37, _e38));
    _ = y_1;
    _ = x_1;
    let _e43: f32 = y_1;
    let _e44: f32 = x_1;
    _ = atan2(_e43, _e44);
    _ = s;
    _ = x_1;
    _ = y_1;
    let _e52: f32 = x_1;
    let _e53: f32 = y_1;
    _ = y_1;
    _ = x_1;
    let _e58: f32 = y_1;
    let _e59: f32 = x_1;
    let _e61: bool = s;
    return select(((3.141592025756836 / 2.0) - atan2(_e52, _e53)), atan2(_e58, _e59), _e61);
}

fn main_1() {
    var fsun: vec3<f32>;
    var pos: vec3<f32>;
    var longitude: f32;
    var color_1: vec3<f32>;

    let _e21: vec3<f32> = global.params.sun;
    fsun = _e21;
    let _e23: vec3<f32> = fsun;
    _ = _e23.yz;
    let _e25: vec3<f32> = fsun;
    let _e26: vec2<f32> = _e25.zy;
    fsun.y = _e26.x;
    fsun.z = _e26.y;
    let _e31: vec3<f32> = in_pos_1;
    _ = _e31.xyz;
    let _e33: vec3<f32> = in_pos_1;
    pos = normalize(_e33.xyz);
    let _e37: vec3<f32> = pos;
    _ = _e37.yz;
    let _e39: vec3<f32> = pos;
    let _e40: vec2<f32> = _e39.zy;
    pos.y = _e40.x;
    pos.z = _e40.y;
    let _e45: vec3<f32> = pos;
    _ = _e45.x;
    let _e47: vec3<f32> = pos;
    _ = _e47.z;
    let _e49: vec3<f32> = pos;
    let _e51: vec3<f32> = pos;
    let _e53: f32 = atan2_(_e49.x, _e51.z);
    longitude = _e53;
    let _e57: i32 = global.params.realistic_sky;
    if (_e57 != 0) {
        {
            _ = pos;
            _ = fsun;
            let _e62: vec3<f32> = pos;
            let _e63: vec3<f32> = fsun;
            let _e64: vec3<f32> = atmosphere(_e62, _e63);
            color_1 = _e64;
        }
    } else {
        {
            let _e66: vec3<f32> = fsun;
            let _e73: vec3<f32> = pos;
            _ = _e73.y;
            let _e76: vec3<f32> = pos;
            _ = vec2<f32>((0.5 - (_e66.y * 0.5)), (1.0 - max(0.009999999776482582, _e76.y)));
            let _e82: vec3<f32> = fsun;
            let _e89: vec3<f32> = pos;
            _ = _e89.y;
            let _e92: vec3<f32> = pos;
            let _e97: vec4<f32> = textureSample(t_gradientsky, s_gradientsky, vec2<f32>((0.5 - (_e82.y * 0.5)), (1.0 - max(0.009999999776482582, _e92.y))));
            color_1 = _e97.xyz;
        }
    }
    let _e99: vec3<f32> = color_1;
    let _e100: vec3<f32> = pos;
    _ = (_e100.y + 0.10000000149011612);
    let _e105: vec3<f32> = pos;
    let _e113: f32 = longitude;
    let _e114: vec3<f32> = pos;
    _ = vec2<f32>(_e113, _e114.y);
    let _e117: f32 = longitude;
    let _e118: vec3<f32> = pos;
    let _e121: vec4<f32> = textureSample(t_starfield, s_starfield, vec2<f32>(_e117, _e118.y));
    color_1 = (_e99 + ((max((_e105.y + 0.10000000149011612), 0.0) * 5.0) * _e121.xyz));
    let _e125: vec3<f32> = color_1;
    let _e126: vec3<f32> = pos;
    _ = _e126.y;
    let _e129: vec3<f32> = pos;
    _ = fsun;
    _ = pos;
    let _e139: vec3<f32> = fsun;
    let _e140: vec3<f32> = pos;
    _ = dot(_e139, _e140);
    _ = fsun;
    _ = pos;
    let _e146: vec3<f32> = fsun;
    let _e147: vec3<f32> = pos;
    color_1 = (_e125 + vec3<f32>(((max(_e129.y, 0.0) * 10000.0) * smoothstep(0.9999300241470337, 1.0, dot(_e146, _e147)))));
    let _e153: vec4<f32> = out_color;
    _ = _e153.xyz;
    let _e156: vec3<f32> = color_1;
    _ = -(_e156);
    let _e158: vec3<f32> = color_1;
    let _e163: f32 = dither();
    let _e165: vec3<f32> = ((vec3<f32>(1.0) - exp(-(_e158))) + vec3<f32>(_e163));
    out_color.x = _e165.x;
    out_color.y = _e165.y;
    out_color.z = _e165.z;
    out_color.w = 1.0;
    return;
}

@fragment 
fn main(@location(0) in_pos: vec3<f32>, @builtin(position) param: vec4<f32>) -> FragmentOutput {
    in_pos_1 = in_pos;
    gl_FragCoord = param;
    _ = (&global.params);
    _ = vec3<f32>(f32(0), 6372000.0, f32(0));
    _ = vec3<f32>(5.500000042957254e-6, 1.2999999853491317e-5, 2.2399999579647556e-5);
    main_1();
    let _e58: vec4<f32> = out_color;
    return FragmentOutput(_e58);
}
