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
var<private> gl_FragCoord: vec4<f32>;

fn dither() -> f32 {
    var color: f32;

    let _e16: vec4<f32> = gl_FragCoord;
    let _e21: vec4<f32> = gl_FragCoord;
    let _e26: vec4<f32> = textureSample(t_bnoise, s_bnoise, (_e21.xy / vec2<f32>(512.0)));
    color = _e26.x;
    let _e29: f32 = color;
    return ((_e29 - 0.5) / 512.0);
}

fn sampleShadow() -> f32 {
    var light_local: vec4<f32>;
    var corrected: vec3<f32>;
    var v: f32;

    let _e16: RenderParams = global.params;
    let _e18: vec3<f32> = in_wpos_1;
    light_local = (_e16.sunproj * vec4<f32>(_e18.x, _e18.y, _e18.z, f32(1)));
    let _e27: vec4<f32> = light_local;
    let _e29: vec4<f32> = light_local;
    corrected = (((_e27.xyz / vec3<f32>(_e29.w)) * vec3<f32>(0.5, -(0.5), 1.0)) + vec3<f32>(0.5, 0.5, 0.0));
    let _e46: vec3<f32> = corrected;
    let _e49: f32 = textureSampleCompare(t_sun_smap, s_sun_smap, _e46.xy, _e46.z);
    v = _e49;
    let _e51: vec4<f32> = light_local;
    if ((_e51.z >= 1.0)) {
        {
            return 1.0;
        }
    }
    let _e58: vec4<f32> = light_local;
    let _e60: vec4<f32> = light_local;
    let _e62: vec4<f32> = light_local;
    let _e64: vec4<f32> = light_local;
    let _e69: vec4<f32> = light_local;
    let _e71: vec4<f32> = light_local;
    let _e73: vec4<f32> = light_local;
    let _e75: vec4<f32> = light_local;
    let _e81: f32 = v;
    let _e84: vec4<f32> = light_local;
    let _e86: vec4<f32> = light_local;
    let _e88: vec4<f32> = light_local;
    let _e90: vec4<f32> = light_local;
    let _e95: vec4<f32> = light_local;
    let _e97: vec4<f32> = light_local;
    let _e99: vec4<f32> = light_local;
    let _e101: vec4<f32> = light_local;
    return mix(_e81, f32(1), clamp(dot(_e99.xy, _e101.xy), 0.0, 1.0));
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

    let _e17: vec2<f32> = in_uv_1;
    let _e18: vec4<f32> = textureSample(t_albedo, s_albedo, _e17);
    albedo = _e18;
    let _e23: RenderParams = global.params;
    if ((_e23.ssao_enabled != 0)) {
        {
            let _e27: vec4<f32> = gl_FragCoord;
            let _e29: RenderParams = global.params;
            let _e32: vec4<f32> = gl_FragCoord;
            let _e34: RenderParams = global.params;
            let _e37: vec4<f32> = textureSample(t_ssao, s_ssao, (_e32.xy / _e34.viewport));
            ssao = _e37.x;
        }
    }
    let _e42: RenderParams = global.params;
    if ((_e42.shadow_mapping_enabled != 0)) {
        {
            let _e46: f32 = sampleShadow();
            shadow_v = _e46;
        }
    }
    let _e48: vec3<f32> = in_normal_1;
    normal = normalize(_e48);
    let _e51: RenderParams = global.params;
    cam = _e51.cam_pos.xyz;
    let _e55: RenderParams = global.params;
    L = _e55.sun;
    let _e59: vec3<f32> = normal;
    let _e64: vec3<f32> = normal;
    let _e65: vec3<f32> = L;
    let _e68: vec3<f32> = L;
    let _e71: vec3<f32> = normal;
    let _e76: vec3<f32> = normal;
    let _e77: vec3<f32> = L;
    let _e80: vec3<f32> = L;
    R = normalize((((f32(2) * _e71) * dot(_e76, _e77)) - _e80));
    let _e84: vec3<f32> = cam;
    let _e85: vec3<f32> = in_wpos_1;
    let _e87: vec3<f32> = cam;
    let _e88: vec3<f32> = in_wpos_1;
    V = normalize((_e87 - _e88));
    let _e94: vec3<f32> = R;
    let _e95: vec3<f32> = V;
    let _e101: vec3<f32> = R;
    let _e102: vec3<f32> = V;
    specular = clamp(dot(_e101, _e102), 0.0, 1.0);
    let _e110: f32 = specular;
    specular = pow(_e110, f32(5));
    let _e115: RenderParams = global.params;
    let _e117: vec3<f32> = normal;
    let _e118: RenderParams = global.params;
    let _e124: RenderParams = global.params;
    let _e126: vec3<f32> = normal;
    let _e127: RenderParams = global.params;
    sun_contrib = clamp(dot(_e126, _e127.sun), 0.0, 1.0);
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
    let _e157: RenderParams = global.params;
    let _e160: vec4<f32> = c;
    final_rgb = (_e155 + (_e156 * (_e157.sun_col.xyz * _e160.xyz)));
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

[[stage(fragment)]]
fn main([[location(0)]] in_tint: vec4<f32>, [[location(1)]] in_normal: vec3<f32>, [[location(2)]] in_wpos: vec3<f32>, [[location(3)]] in_uv: vec2<f32>, [[builtin(position)]] param: vec4<f32>) -> FragmentOutput {
    in_tint_1 = in_tint;
    in_normal_1 = in_normal;
    in_wpos_1 = in_wpos;
    in_uv_1 = in_uv;
    gl_FragCoord = param;
    main_1();
    let _e39: vec4<f32> = out_color;
    return FragmentOutput(_e39);
}
