struct RenderParams {
    invproj: mat4x4<f32>;
    sunproj: mat4x4<f32>;
    cam_pos: vec4<f32>;
    cam_dir: vec4<f32>;
    sun: vec3<f32>;
    sun_col: vec4<f32>;
    grass_col: vec4<f32>;
    sand_col: vec4<f32>;
    sea_col: vec4<f32>;
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

struct FragmentOutput {
    [[location(0)]] out_color: vec4<f32>;
};

var<private> in_normal_1: vec3<f32>;
var<private> in_wpos_1: vec3<f32>;
var<private> out_color: vec4<f32>;
[[group(1), binding(0)]]
var<uniform> global: Uni;
[[group(2), binding(0)]]
var t_terraindata: texture_2d<f32>;
[[group(2), binding(1)]]
var s_terraindata: sampler;
[[group(3), binding(0)]]
var t_ssao: texture_2d<f32>;
[[group(3), binding(1)]]
var s_ssao: sampler;
[[group(3), binding(2)]]
var t_bnoise: texture_2d<f32>;
[[group(3), binding(3)]]
var s_bnoise: sampler;
[[group(3), binding(4)]]
var t_sun_smap: texture_depth_2d;
[[group(3), binding(5)]]
var s_sun_smap: sampler_comparison;
var<private> gl_FragCoord: vec4<f32>;

fn dither() -> f32 {
    var color: f32;

    let _e14: vec4<f32> = gl_FragCoord;
    let _e19: vec4<f32> = gl_FragCoord;
    let _e24: vec4<f32> = textureSample(t_bnoise, s_bnoise, (_e19.xy / vec2<f32>(512.0)));
    color = _e24.x;
    let _e27: f32 = color;
    return ((_e27 - 0.5) / 512.0);
}

fn sampleShadow() -> f32 {
    var light_local: vec4<f32>;
    var corrected: vec3<f32>;
    var v: f32;

    let _e14: RenderParams = global.params;
    let _e16: vec3<f32> = in_wpos_1;
    light_local = (_e14.sunproj * vec4<f32>(_e16.x, _e16.y, _e16.z, f32(1)));
    let _e25: vec4<f32> = light_local;
    let _e27: vec4<f32> = light_local;
    corrected = (((_e25.xyz / vec3<f32>(_e27.w)) * vec3<f32>(0.5, -(0.5), 1.0)) + vec3<f32>(0.5, 0.5, 0.0));
    let _e44: vec3<f32> = corrected;
    let _e47: f32 = textureSampleCompare(t_sun_smap, s_sun_smap, _e44.xy, _e44.z);
    v = _e47;
    let _e49: vec4<f32> = light_local;
    if ((_e49.z >= 1.0)) {
        {
            return 1.0;
        }
    }
    let _e56: vec4<f32> = light_local;
    let _e58: vec4<f32> = light_local;
    let _e60: vec4<f32> = light_local;
    let _e62: vec4<f32> = light_local;
    let _e67: vec4<f32> = light_local;
    let _e69: vec4<f32> = light_local;
    let _e71: vec4<f32> = light_local;
    let _e73: vec4<f32> = light_local;
    let _e79: f32 = v;
    let _e82: vec4<f32> = light_local;
    let _e84: vec4<f32> = light_local;
    let _e86: vec4<f32> = light_local;
    let _e88: vec4<f32> = light_local;
    let _e93: vec4<f32> = light_local;
    let _e95: vec4<f32> = light_local;
    let _e97: vec4<f32> = light_local;
    let _e99: vec4<f32> = light_local;
    return mix(_e79, f32(1), clamp(dot(_e97.xy, _e99.xy), 0.0, 1.0));
}

fn main_1() {
    var ssao: f32 = 1.0;
    var shadow_v: f32 = 1.0;
    var c: vec4<f32>;
    var v_1: f32;
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

    let _e17: RenderParams = global.params;
    if ((_e17.ssao_enabled != 0)) {
        {
            let _e21: vec4<f32> = gl_FragCoord;
            let _e23: RenderParams = global.params;
            let _e26: vec4<f32> = gl_FragCoord;
            let _e28: RenderParams = global.params;
            let _e31: vec4<f32> = textureSample(t_ssao, s_ssao, (_e26.xy / _e28.viewport));
            ssao = _e31.x;
        }
    }
    let _e36: RenderParams = global.params;
    if ((_e36.shadow_mapping_enabled != 0)) {
        {
            let _e40: f32 = sampleShadow();
            shadow_v = _e40;
        }
    }
    let _e41: RenderParams = global.params;
    c = _e41.grass_col;
    let _e44: vec3<f32> = in_wpos_1;
    let _e48: vec3<f32> = in_wpos_1;
    let _e53: vec3<f32> = in_wpos_1;
    let _e57: vec3<f32> = in_wpos_1;
    let _e64: vec3<f32> = in_wpos_1;
    let _e68: vec3<f32> = in_wpos_1;
    let _e73: vec3<f32> = in_wpos_1;
    let _e77: vec3<f32> = in_wpos_1;
    v_1 = ((floor((_e68.x * 0.009999999776482582)) + floor((_e77.y * 0.009999999776482582))) % 2.0);
    let _e86: vec4<f32> = c;
    let _e94: f32 = v_1;
    c = (_e86 + vec4<f32>(0.0, (0.019999999552965164 * smoothStep(0.9900000095367432, 1.0099999904632568, _e94)), 0.0, 0.0));
    let _e101: RenderParams = global.params;
    let _e107: vec3<f32> = in_wpos_1;
    let _e112: vec3<f32> = in_wpos_1;
    let _e115: RenderParams = global.params;
    let _e117: vec4<f32> = c;
    let _e121: vec3<f32> = in_wpos_1;
    let _e126: vec3<f32> = in_wpos_1;
    c = mix(_e115.sand_col, _e117, vec4<f32>(smoothStep(-(5.0), 0.0, _e126.z)));
    let _e131: RenderParams = global.params;
    let _e138: vec3<f32> = in_wpos_1;
    let _e144: vec3<f32> = in_wpos_1;
    let _e147: RenderParams = global.params;
    let _e149: vec4<f32> = c;
    let _e154: vec3<f32> = in_wpos_1;
    let _e160: vec3<f32> = in_wpos_1;
    c = mix(_e147.sea_col, _e149, vec4<f32>(smoothStep(-(25.0), -(20.0), _e160.z)));
    let _e166: vec3<f32> = in_normal_1;
    normal = normalize(_e166);
    let _e169: RenderParams = global.params;
    cam = _e169.cam_pos.xyz;
    let _e173: RenderParams = global.params;
    L = _e173.sun;
    let _e177: vec3<f32> = normal;
    let _e182: vec3<f32> = normal;
    let _e183: vec3<f32> = L;
    let _e186: vec3<f32> = L;
    let _e189: vec3<f32> = normal;
    let _e194: vec3<f32> = normal;
    let _e195: vec3<f32> = L;
    let _e198: vec3<f32> = L;
    R = normalize((((f32(2) * _e189) * dot(_e194, _e195)) - _e198));
    let _e202: vec3<f32> = cam;
    let _e203: vec3<f32> = in_wpos_1;
    let _e205: vec3<f32> = cam;
    let _e206: vec3<f32> = in_wpos_1;
    V = normalize((_e205 - _e206));
    let _e212: vec3<f32> = R;
    let _e213: vec3<f32> = V;
    let _e219: vec3<f32> = R;
    let _e220: vec3<f32> = V;
    specular = clamp(dot(_e219, _e220), 0.0, 1.0);
    let _e228: f32 = specular;
    specular = pow(_e228, f32(2));
    let _e233: RenderParams = global.params;
    let _e235: vec3<f32> = normal;
    let _e236: RenderParams = global.params;
    let _e242: RenderParams = global.params;
    let _e244: vec3<f32> = normal;
    let _e245: RenderParams = global.params;
    sun_contrib = clamp(dot(_e244, _e245.sun), 0.0, 1.0);
    let _e253: vec4<f32> = c;
    ambiant = (0.15000000596046448 * _e253.xyz);
    let _e258: f32 = sun_contrib;
    let _e261: f32 = specular;
    let _e264: f32 = shadow_v;
    sun = (((0.8500000238418579 * _e258) + (0.5 * _e261)) * _e264);
    let _e267: vec3<f32> = ambiant;
    final_rgb = _e267;
    let _e269: vec3<f32> = final_rgb;
    let _e270: f32 = sun;
    let _e271: RenderParams = global.params;
    let _e274: vec4<f32> = c;
    final_rgb = (_e269 + (_e270 * (_e271.sun_col.xyz * _e274.xyz)));
    let _e279: vec3<f32> = final_rgb;
    let _e280: f32 = ssao;
    final_rgb = (_e279 * _e280);
    let _e282: vec3<f32> = final_rgb;
    let _e283: f32 = dither();
    final_rgb = (_e282 + vec3<f32>(_e283));
    let _e286: vec3<f32> = final_rgb;
    let _e287: vec4<f32> = c;
    out_color = vec4<f32>(_e286.x, _e286.y, _e286.z, _e287.w);
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] in_normal: vec3<f32>, [[location(1)]] in_wpos: vec3<f32>, [[builtin(position)]] param: vec4<f32>) -> FragmentOutput {
    in_normal_1 = in_normal;
    in_wpos_1 = in_wpos;
    gl_FragCoord = param;
    main_1();
    let _e31: vec4<f32> = out_color;
    return FragmentOutput(_e31);
}
