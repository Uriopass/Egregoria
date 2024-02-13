#include "../render_params.wgsl"
#include "../shadow.wgsl"
#include "../pbr/render.wgsl"
#include "../atmosphere.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

struct ChunkData {
    lod: u32,
    resolution: u32,
}

@group(0) @binding(0) var<uniform> params: RenderParams;

@group(1) @binding(4) var t_grass: texture_2d<f32>;
@group(1) @binding(5) var s_grass: sampler;
@group(1) @binding(6) var t_cliff: texture_2d<f32>;
@group(1) @binding(7) var s_cliff: sampler;
@group(1) @binding(8) var<uniform> cdata: ChunkData;

@group(2) @binding(0) var t_bnoise: texture_2d<f32>;
@group(2) @binding(1) var s_bnoise: sampler;
@group(2) @binding(2) var t_sun_smap: texture_depth_2d_array;
@group(2) @binding(3) var s_sun_smap: sampler_comparison;
@group(2) @binding(4) var t_diffuse_irradiance: texture_cube<f32>;
@group(2) @binding(5) var s_diffuse_irradiance: sampler;
@group(2) @binding(6) var t_prefilter_specular: texture_cube<f32>;
@group(2) @binding(7) var s_prefilter_specular: sampler;
@group(2) @binding(8) var t_brdf_lut: texture_2d<f32>;
@group(2) @binding(9) var s_brdf_lut: sampler;
@group(2) @binding(10) var t_ssao: texture_2d<f32>;
@group(2) @binding(11) var s_ssao: sampler;
@group(2) @binding(12) var t_fog: texture_2d<f32>;
@group(2) @binding(13) var s_fog: sampler;
@group(2) @binding(14) var t_lightdata: texture_2d<u32>;
@group(2) @binding(15) var s_lightdata: sampler;
@group(2) @binding(16) var t_lightdata2: texture_2d<u32>;
@group(2) @binding(17) var s_lightdata2: sampler;

#ifdef TERRAIN_GRID
fn grid(in_wpos: vec3<f32>, wpos_fwidth_x: f32) -> f32 {
    let level: f32 = wpos_fwidth_x * 20.0;

    let max_w: f32 = level * 100.0;
    let min_w: f32 = level;

    var curgrid: vec2<f32> = in_wpos.xy / 10000.0;
    var isIn: f32 = 0.0;

    var w: f32 = 10000.0;
    while (w > max_w) {
        w /= 5.0;
        curgrid *= 5.0;
    }

    while (w > min_w) {
        let moved: vec2<f32> = fract(curgrid);
        let v: f32 = min(min(moved.x, moved.y), min(1.0 - moved.x, 1.0 - moved.y));

        let threshold1: f32 = smoothstep(0.004, 0.00415, v);
        let threshold2: f32 = smoothstep(max_w * 0.5, max_w, w);
        let isOk: f32 = (1.0 - threshold1) * 2.0 * (1.0 - threshold2);

        isIn = max(isIn, isOk);
        w /= 5.0;
        curgrid *= 5.0;
    }

    return isIn;
}
#endif

fn hash4(p: vec2<f32>) -> vec4<f32> { return fract(sin(vec4( 1.0+dot(p,vec2(37.0,17.0)),
                                              2.0+dot(p,vec2(11.0,47.0)),
                                              3.0+dot(p,vec2(41.0,29.0)),
                                              4.0+dot(p,vec2(23.0,31.0))))*103.0); }

// taken from https://iquilezles.org/articles/texturerepetition/
fn textureNoTile(tex: texture_2d<f32>, samp: sampler, uv: vec2<f32>) -> vec4<f32> {
    let iuv: vec2<f32> = floor( uv );
    let fuv: vec2<f32> = fract( uv );

    // generate per-tile transform
    var ofa: vec4<f32> = hash4( iuv + vec2(0.0,0.0) );
    var ofb: vec4<f32> = hash4( iuv + vec2(1.0,0.0) );
    var ofc: vec4<f32> = hash4( iuv + vec2(0.0,1.0) );
    var ofd: vec4<f32> = hash4( iuv + vec2(1.0,1.0) );

    let ddx: vec2<f32> = dpdxCoarse(uv);
    let ddy: vec2<f32> = dpdyCoarse(uv);

    // transform per-tile uvs
    ofa.z = sign(ofa.z - 0.5);
    ofa.w = sign(ofa.w - 0.5);
    ofb.z = sign(ofb.z - 0.5);
    ofb.w = sign(ofb.w - 0.5);
    ofc.z = sign(ofc.z - 0.5);
    ofc.w = sign(ofc.w - 0.5);
    ofd.z = sign(ofd.z - 0.5);
    ofd.w = sign(ofd.w - 0.5);

    // uv's, and derivarives (for correct mipmapping)
    let uva: vec2<f32> = uv*ofa.zw + ofa.xy; let ddxa: vec2<f32> = ddx*ofa.zw; let ddya: vec2<f32> = ddy*ofa.zw;
    let uvb: vec2<f32> = uv*ofb.zw + ofb.xy; let ddxb: vec2<f32> = ddx*ofb.zw; let ddyb: vec2<f32> = ddy*ofb.zw;
    let uvc: vec2<f32> = uv*ofc.zw + ofc.xy; let ddxc: vec2<f32> = ddx*ofc.zw; let ddyc: vec2<f32> = ddy*ofc.zw;
    let uvd: vec2<f32> = uv*ofd.zw + ofd.xy; let ddxd: vec2<f32> = ddx*ofd.zw; let ddyd: vec2<f32> = ddy*ofd.zw;

    // fetch and blend
    let b: vec2<f32> = smoothstep(vec2(0.25),vec2(0.75),fuv);

    return mix( mix( textureSampleGrad( tex, samp, uva, ddxa, ddya ),
                     textureSampleGrad( tex, samp, uvb, ddxb, ddyb ), b.x ),
                mix( textureSampleGrad( tex, samp, uvc, ddxc, ddyc ),
                     textureSampleGrad( tex, samp, uvd, ddxd, ddyd ), b.x), b.y );
}

@fragment
fn frag(@builtin(position) position: vec4<f32>,
        @location(0) in_normal: vec3<f32>,
        @location(1) in_wpos: vec3<f32>,
#ifdef DEBUG
        @location(2) debug: f32,
#endif
        ) -> FragmentOutput {
    var ssao = 1.0;
    #ifdef SSAO
    ssao = textureSample(t_ssao, s_ssao, position.xy / params.viewport).r;
    #endif

    var shadow_v: f32 = 1.0;
    if (params.shadow_mapping_resolution != 0) {
        shadow_v = sampleShadow(in_wpos);
    }

    var c: vec3<f32> = vec3(0.0, 0.0, 0.0);
    let wcliffN = pow(abs(in_normal.x), 16.0);
    let wcliffE = pow(abs(in_normal.y), 16.0);
    let wgrass  = pow(abs(in_normal.z)*0.8, 16.0);

    let sum = wgrass + wcliffE + wcliffN;

    let min_contribution = 0.01 * sum;

    // tri-planar mapping
    if (wgrass  > min_contribution) { c += wgrass  * textureNoTile(t_grass, s_grass, in_wpos.xy / 200.0).rgb; }
    if (wcliffE > min_contribution) { c += wcliffE * textureSample(t_cliff, s_cliff, in_wpos.xz / 100.0).rgb; }
    if (wcliffN > min_contribution) { c += wcliffN * textureSample(t_cliff, s_cliff, in_wpos.yz / 100.0).rgb; }

    c = c / sum;

    #ifdef TERRAIN_GRID
    c.g += grid(in_wpos, fwidthCoarse(in_wpos.x)) * 0.015;
    #endif

    c = mix(params.sand_col.rgb, c, smoothstep(-5.0, 0.0, in_wpos.z));
    c = mix(params.sea_col.rgb, c, smoothstep(-25.0, -20.0, in_wpos.z));

    let irradiance_diffuse: vec3<f32> = textureSample(t_diffuse_irradiance, s_diffuse_irradiance, in_normal).rgb;
    let V_denorm: vec3<f32> = params.cam_pos.xyz - in_wpos;
    let dist: f32 = length(V_denorm);
    let V: vec3<f32> = V_denorm / dist;
    let F0: vec3<f32> = vec3(0.01);
    let roughness: f32 = 1.3; // avoid specular highlights which look weird on terrain
    let normal: vec3<f32> = normalize(in_normal);
    let F_spec: vec3<f32> = F0; // simplified with constant folding: fresnelSchlickRoughness(max(dot(normal, V), 0.0), F0, roughness);

    #ifdef DEBUG
    c = 0.05 * vec3(0.0, 0.0, debug);
    #endif

    if (params.terraforming_mode_radius > 0.0) {
        let dist_to_mouse = length(params.unproj_pos - in_wpos.xy);
        var fw = fwidthCoarse(in_wpos.z) * 2.5;

        let alpha = smoothstep(params.terraforming_mode_radius, params.terraforming_mode_radius*0.4, dist_to_mouse);
        let alpha4 = smoothstep(fw, 0.0, abs((in_wpos.z % 10.0) - 5.0)) * 0.1;
        let alpha5 = smoothstep(fw, 0.0, abs((in_wpos.z % 50.0) - 25.0)) * 0.1;

        c = mix(c, vec3(0.7, 0.4, 0.2), alpha * 0.15);
        c = mix(c, vec3(0.0, 0.0, 0.0), alpha4 + alpha5);
    }

    var fog = vec3(0.0);
    #ifdef FOG
    var fogdist: vec4<f32> = textureSampleLevel(t_fog, s_fog, position.xy / params.viewport, 0.0);

    if (abs(fogdist.a - dist) > 150.0) {
        #ifdef FOG_DEBUG
        fog = vec3(1.0);
        #else
        fog = atmosphere(-V, params.sun, dist * 0.2);
        #endif
    } else {
        fog = fogdist.rgb;
    }
    #endif

    let lightdata = get_lightdata(t_lightdata, t_lightdata2, in_wpos);

    let final_rgb: vec3<f32> = render(params.sun,
                                      V,
                                      position.xy,
                                      normal,
                                      c,
                                      F0,
                                      F_spec,
                                      params.sun_col.rgb,
                                      irradiance_diffuse,
                                      vec3(0.0),
                                      0.0,
                                      roughness,
                                      shadow_v,
                                      ssao,
                                      lightdata,
                                      in_wpos,
                                      fog
                                      );
    return FragmentOutput(vec4(final_rgb, 1.0));
}
