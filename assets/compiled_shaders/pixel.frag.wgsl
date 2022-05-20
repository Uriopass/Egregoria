struct RenderParams {
    invproj: mat4x4<f32>;
    sunproj: mat4x4<f32>;
    cam_pos: vec4<f32>;
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

struct Uni {
    params: RenderParams;
};

struct FragmentOutput {
    [[location(0)]] out_color: vec4<f32>;
};

var<private> in_tint_1: vec4<f32>;
var<private> in_normal_1: vec3<f32>;
var<private> in_wpos_1: vec3<f32>;
var<private> in_uv_1: vec2<f32>;
var<private> out_color: vec4<f32>;
[[group(1), binding(0)]]
var<uniform> global: Uni;
[[group(2), binding(0)]]
var t_albedo: texture_2d<f32>;
[[group(2), binding(1)]]
var s_albedo: sampler;
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
[[group(3), binding(6)]]
var t_quadlights: texture_2d<f32>;
[[group(3), binding(7)]]
var s_quadlights: sampler;
var<private> gl_FragCoord: vec4<f32>;

fn dither() -> f32 {
    var color: f32;

    let _e18: vec4<f32> = gl_FragCoord;
    let _e23: vec4<f32> = gl_FragCoord;
    let _e28: vec4<f32> = textureSample(t_bnoise, s_bnoise, (_e23.xy / vec2<f32>(512.0)));
    color = _e28.x;
    let _e31: f32 = color;
    return ((_e31 - 0.5) / 512.0);
}

fn sampleShadow() -> f32 {
    var light_local: vec4<f32>;
    var corrected: vec3<f32>;
    var v: f32;

    let _e18: RenderParams = global.params;
    let _e20: vec3<f32> = in_wpos_1;
    light_local = (_e18.sunproj * vec4<f32>(_e20.x, _e20.y, _e20.z, f32(1)));
    let _e29: vec4<f32> = light_local;
    let _e31: vec4<f32> = light_local;
    corrected = (((_e29.xyz / vec3<f32>(_e31.w)) * vec3<f32>(0.5, -(0.5), 1.0)) + vec3<f32>(0.5, 0.5, 0.0));
    let _e48: vec3<f32> = corrected;
    let _e51: f32 = textureSampleCompare(t_sun_smap, s_sun_smap, _e48.xy, _e48.z);
    v = _e51;
    let _e53: vec4<f32> = light_local;
    if ((_e53.z >= 1.0)) {
        {
            return 1.0;
        }
    }
    let _e60: vec4<f32> = light_local;
    let _e62: vec4<f32> = light_local;
    let _e64: vec4<f32> = light_local;
    let _e66: vec4<f32> = light_local;
    let _e71: vec4<f32> = light_local;
    let _e73: vec4<f32> = light_local;
    let _e75: vec4<f32> = light_local;
    let _e77: vec4<f32> = light_local;
    let _e83: f32 = v;
    let _e86: vec4<f32> = light_local;
    let _e88: vec4<f32> = light_local;
    let _e90: vec4<f32> = light_local;
    let _e92: vec4<f32> = light_local;
    let _e97: vec4<f32> = light_local;
    let _e99: vec4<f32> = light_local;
    let _e101: vec4<f32> = light_local;
    let _e103: vec4<f32> = light_local;
    return mix(_e83, f32(1), clamp(dot(_e101.xy, _e103.xy), 0.0, 1.0));
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

    let _e19: vec2<f32> = in_uv_1;
    let _e20: vec4<f32> = textureSample(t_albedo, s_albedo, _e19);
    albedo = _e20;
    let _e25: RenderParams = global.params;
    if ((_e25.ssao_enabled != 0)) {
        {
            let _e29: vec4<f32> = gl_FragCoord;
            let _e31: RenderParams = global.params;
            let _e34: vec4<f32> = gl_FragCoord;
            let _e36: RenderParams = global.params;
            let _e39: vec4<f32> = textureSample(t_ssao, s_ssao, (_e34.xy / _e36.viewport));
            ssao = _e39.x;
        }
    }
    let _e44: RenderParams = global.params;
    if ((_e44.shadow_mapping_enabled != 0)) {
        {
            let _e48: f32 = sampleShadow();
            shadow_v = _e48;
        }
    }
    let _e50: vec3<f32> = in_normal_1;
    normal = normalize(_e50);
    let _e53: RenderParams = global.params;
    cam = _e53.cam_pos.xyz;
    let _e57: RenderParams = global.params;
    L = _e57.sun;
    let _e61: vec3<f32> = normal;
    let _e66: vec3<f32> = normal;
    let _e67: vec3<f32> = L;
    let _e70: vec3<f32> = L;
    let _e73: vec3<f32> = normal;
    let _e78: vec3<f32> = normal;
    let _e79: vec3<f32> = L;
    let _e82: vec3<f32> = L;
    R = normalize((((f32(2) * _e73) * dot(_e78, _e79)) - _e82));
    let _e86: vec3<f32> = cam;
    let _e87: vec3<f32> = in_wpos_1;
    let _e89: vec3<f32> = cam;
    let _e90: vec3<f32> = in_wpos_1;
    V = normalize((_e89 - _e90));
    let _e96: vec3<f32> = R;
    let _e97: vec3<f32> = V;
    let _e103: vec3<f32> = R;
    let _e104: vec3<f32> = V;
    specular = clamp(dot(_e103, _e104), 0.0, 1.0);
    let _e112: f32 = specular;
    specular = pow(_e112, f32(5));
    let _e117: RenderParams = global.params;
    let _e119: vec3<f32> = normal;
    let _e120: RenderParams = global.params;
    let _e126: RenderParams = global.params;
    let _e128: vec3<f32> = normal;
    let _e129: RenderParams = global.params;
    sun_contrib = clamp(dot(_e128, _e129.sun), 0.0, 1.0);
    let _e136: vec4<f32> = in_tint_1;
    let _e137: vec4<f32> = albedo;
    c = (_e136 * _e137);
    let _e141: vec4<f32> = c;
    ambiant = (0.15000000596046448 * _e141.xyz);
    let _e146: f32 = sun_contrib;
    let _e149: f32 = specular;
    let _e152: f32 = shadow_v;
    sun = (((0.8500000238418579 * _e146) + (0.5 * _e149)) * _e152);
    let _e155: vec3<f32> = ambiant;
    final_rgb = _e155;
    let _e157: vec3<f32> = final_rgb;
    let _e158: f32 = sun;
    let _e159: RenderParams = global.params;
    let _e162: vec4<f32> = c;
    final_rgb = (_e157 + (_e158 * (_e159.sun_col.xyz * _e162.xyz)));
    let _e167: vec3<f32> = final_rgb;
    let _e168: f32 = ssao;
    final_rgb = (_e167 * _e168);
    let _e170: vec3<f32> = final_rgb;
    let _e171: f32 = dither();
    final_rgb = (_e170 + vec3<f32>(_e171));
    let _e174: vec3<f32> = final_rgb;
    let _e175: vec4<f32> = c;
    out_color = vec4<f32>(_e174.x, _e174.y, _e174.z, _e175.w);
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] in_tint: vec4<f32>, [[location(1)]] in_normal: vec3<f32>, [[location(2)]] in_wpos: vec3<f32>, [[location(3)]] in_uv: vec2<f32>, [[builtin(position)]] param: vec4<f32>) -> FragmentOutput {
    in_tint_1 = in_tint;
    in_normal_1 = in_normal;
    in_wpos_1 = in_wpos;
    in_uv_1 = in_uv;
    gl_FragCoord = param;
    main_1();
    let _e43: vec4<f32> = out_color;
    return FragmentOutput(_e43);
}
