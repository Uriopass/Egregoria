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

struct FragmentOutput {
    [[location(0)]] out_ssao: f32;
};

var<private> in_uv_1: vec2<f32>;
var<private> out_ssao: f32;
[[group(0), binding(0)]]
var t_depth: texture_multisampled_2d<f32>;
[[group(0), binding(1)]]
var s_depth: sampler;
[[group(1), binding(0)]]
var<uniform> global: Uni;
var<private> PHI: f32 = 1.6180340051651;

fn fastnoise(xy: vec2<f32>, seed: f32) -> f32 {
    var xy_1: vec2<f32>;
    var seed_1: f32;

    xy_1 = xy;
    seed_1 = seed;
    let _e11: vec2<f32> = xy_1;
    let _e12: f32 = PHI;
    let _e15: vec2<f32> = xy_1;
    let _e16: f32 = PHI;
    let _e18: vec2<f32> = xy_1;
    let _e20: f32 = seed_1;
    let _e22: vec2<f32> = xy_1;
    let _e23: f32 = PHI;
    let _e26: vec2<f32> = xy_1;
    let _e27: f32 = PHI;
    let _e29: vec2<f32> = xy_1;
    let _e31: f32 = seed_1;
    let _e34: vec2<f32> = xy_1;
    let _e37: vec2<f32> = xy_1;
    let _e38: f32 = PHI;
    let _e41: vec2<f32> = xy_1;
    let _e42: f32 = PHI;
    let _e44: vec2<f32> = xy_1;
    let _e46: f32 = seed_1;
    let _e48: vec2<f32> = xy_1;
    let _e49: f32 = PHI;
    let _e52: vec2<f32> = xy_1;
    let _e53: f32 = PHI;
    let _e55: vec2<f32> = xy_1;
    let _e57: f32 = seed_1;
    let _e60: vec2<f32> = xy_1;
    return fract((tan((distance((_e52 * _e53), _e55) * _e57)) * _e60.x));
}

fn uv2s(uv: vec2<f32>) -> vec2<f32> {
    var uv_1: vec2<f32>;

    uv_1 = uv;
    let _e9: vec2<f32> = uv_1;
    let _e10: RenderParams = global.params;
    let _e13: vec2<f32> = uv_1;
    let _e14: RenderParams = global.params;
    return round((_e13 * _e14.viewport));
}

fn sample_depth(coords: vec2<i32>) -> f32 {
    var coords_1: vec2<i32>;

    coords_1 = coords;
    let _e11: vec2<i32> = coords_1;
    let _e13: vec4<f32> = textureLoad(t_depth, _e11, 0);
    return _e13.x;
}

fn derivative(c: vec2<i32>, depth: f32) -> vec2<f32> {
    var c_1: vec2<i32>;
    var depth_1: f32;
    var depthx: f32;
    var depthy: f32;

    c_1 = c;
    depth_1 = depth;
    let _e11: vec2<i32> = c_1;
    let _e17: vec2<i32> = c_1;
    let _e23: vec4<f32> = textureLoad(t_depth, (_e17 + vec2<i32>(1, 0)), 0);
    depthx = _e23.x;
    let _e26: vec2<i32> = c_1;
    let _e32: vec2<i32> = c_1;
    let _e38: vec4<f32> = textureLoad(t_depth, (_e32 + vec2<i32>(0, 1)), 0);
    depthy = _e38.x;
    let _e41: f32 = depthx;
    let _e42: f32 = depth_1;
    let _e44: f32 = depthy;
    let _e45: f32 = depth_1;
    return vec2<f32>((_e41 - _e42), (_e44 - _e45));
}

fn main_1() {
    var total_strength: f32;
    var base: f32;
    var falloff: f32;
    var radius: f32;
    var samples: i32;
    var sample_sphere: array<vec3<f32>,16u> = array<vec3<f32>,16u>(vec3<f32>(0.538100004196167, 0.18559999763965607, -0.4318999946117401), vec3<f32>(0.1378999948501587, 0.24860000610351563, 0.4429999887943268), vec3<f32>(0.33709999918937683, 0.5679000020027161, -0.00570000009611249), vec3<f32>(-0.6998999714851379, -0.045099999755620956, -0.0019000000320374966), vec3<f32>(0.06889999657869339, -0.1597999930381775, -0.8547000288963318), vec3<f32>(0.0560000017285347, 0.006899999920278788, -0.1843000054359436), vec3<f32>(-0.014600000344216824, 0.14020000398159027, 0.07620000094175339), vec3<f32>(0.009999999776482582, -0.1923999935388565, -0.03440000116825104), vec3<f32>(-0.357699990272522, -0.5300999879837036, -0.4357999861240387), vec3<f32>(-0.31690001487731934, 0.1062999963760376, 0.015799999237060547), vec3<f32>(0.010300000198185444, -0.586899995803833, 0.004600000102072954), vec3<f32>(-0.08969999849796295, -0.49399998784065247, 0.3287000060081482), vec3<f32>(0.711899995803833, -0.015399999916553497, -0.09179999679327011), vec3<f32>(-0.053300000727176666, 0.05959999933838844, -0.541100025177002), vec3<f32>(0.03519999980926514, -0.06310000270605087, 0.5460000038146973), vec3<f32>(-0.47760000824928284, 0.2847000062465668, -0.02710000053048134));
    var xr: f32;
    var yr: f32;
    var zr: f32;
    var random: vec3<f32>;
    var pos: vec2<i32>;
    var depth_2: f32;
    var derivative_1: vec2<f32>;
    var radius_depth: f32;
    var occlusion: f32 = 0.0;
    var i: i32 = 0;
    var ray: vec3<f32>;
    var off: vec2<f32>;
    var occ_depth: f32;
    var difference: f32;
    var dcorrected: f32;
    var ao: f32;
    var v: f32;

    let _e7: RenderParams = global.params;
    total_strength = _e7.ssao_strength;
    let _e10: RenderParams = global.params;
    base = _e10.ssao_base;
    let _e13: RenderParams = global.params;
    falloff = _e13.ssao_falloff;
    let _e16: RenderParams = global.params;
    radius = _e16.ssao_radius;
    let _e19: RenderParams = global.params;
    samples = _e19.ssao_samples;
    let _e113: vec2<f32> = in_uv_1;
    let _e117: vec2<f32> = in_uv_1;
    let _e121: f32 = fastnoise((_e117 * 1000.0), 1.0);
    xr = _e121;
    let _e123: vec2<f32> = in_uv_1;
    let _e127: vec2<f32> = in_uv_1;
    let _e131: f32 = fastnoise((_e127 * 1000.0), 2.0);
    yr = _e131;
    let _e133: vec2<f32> = in_uv_1;
    let _e137: vec2<f32> = in_uv_1;
    let _e141: f32 = fastnoise((_e137 * 1000.0), 3.0);
    zr = _e141;
    let _e143: f32 = xr;
    let _e144: f32 = yr;
    let _e145: f32 = zr;
    let _e147: f32 = xr;
    let _e148: f32 = yr;
    let _e149: f32 = zr;
    random = normalize(vec3<f32>(_e147, _e148, _e149));
    let _e154: vec2<f32> = in_uv_1;
    let _e155: vec2<f32> = uv2s(_e154);
    pos = vec2<i32>(_e155);
    let _e159: vec2<i32> = pos;
    let _e160: f32 = sample_depth(_e159);
    depth_2 = _e160;
    let _e164: vec2<i32> = pos;
    let _e165: f32 = depth_2;
    let _e166: vec2<f32> = derivative(_e164, _e165);
    derivative_1 = _e166;
    let _e168: f32 = radius;
    let _e169: f32 = depth_2;
    radius_depth = (_e168 / _e169);
    loop {
        let _e176: i32 = i;
        let _e177: i32 = samples;
        if (!((_e176 < _e177))) {
            break;
        }
        {
            let _e183: f32 = radius_depth;
            let _e184: i32 = i;
            let _e188: i32 = i;
            let _e190: vec3<f32> = sample_sphere[_e188];
            let _e191: vec3<f32> = random;
            ray = (_e183 * reflect(_e190, _e191));
            let _e195: vec3<f32> = ray;
            let _e197: vec3<f32> = ray;
            let _e199: vec2<f32> = uv2s(_e197.xy);
            off = _e199;
            let _e201: vec2<i32> = pos;
            let _e202: vec2<f32> = off;
            let _e205: vec2<i32> = pos;
            let _e206: vec2<f32> = off;
            let _e209: f32 = sample_depth((_e205 + vec2<i32>(_e206)));
            occ_depth = _e209;
            let _e211: f32 = depth_2;
            let _e212: f32 = occ_depth;
            difference = (_e211 - _e212);
            let _e215: f32 = difference;
            let _e218: vec2<f32> = off;
            let _e219: vec2<f32> = derivative_1;
            dcorrected = (_e215 + dot(_e218, _e219));
            let _e223: f32 = occlusion;
            let _e225: f32 = falloff;
            let _e229: f32 = falloff;
            let _e230: f32 = falloff;
            let _e233: f32 = dcorrected;
            occlusion = (_e223 + smoothStep(_e229, (_e230 * 2.0), _e233));
        }
        continuing {
            let _e180: i32 = i;
            i = (_e180 + 1);
        }
    }
    let _e237: f32 = total_strength;
    let _e238: f32 = occlusion;
    let _e241: i32 = samples;
    ao = (1.0 - ((_e237 * _e238) * (1.0 / f32(_e241))));
    let _e247: f32 = ao;
    let _e248: f32 = base;
    let _e252: f32 = ao;
    let _e253: f32 = base;
    v = clamp((_e252 + _e253), 0.0, 1.0);
    let _e259: f32 = v;
    out_ssao = _e259;
    return;
}

[[stage(fragment)]]
fn main([[location(0)]] in_uv: vec2<f32>) -> FragmentOutput {
    in_uv_1 = in_uv;
    main_1();
    let _e16: f32 = out_ssao;
    return FragmentOutput(_e16);
}
