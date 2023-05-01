#include "render_params.wgsl"

struct FragmentOutput {
    @location(0) out_color: vec4<f32>,
}

const HAS_METALLIC_ROUGHNESS_TEXTURE: u32 = 1u;
const HAS_NORMAL_MAP: u32 = 2u;

struct MaterialParams {
    flags: u32,
    metallic: f32,
    roughness: f32,
}

@group(1) @binding(0) var<uniform> params: RenderParams;

@group(2) @binding(0) var t_albedo: texture_2d<f32>;
@group(2) @binding(1) var s_albedo: sampler;
@group(2) @binding(2) var<uniform> u_mat: MaterialParams;
@group(2) @binding(3) var t_metallic_roughness: texture_2d<f32>;
@group(2) @binding(4) var s_metallic_rougness: sampler;
@group(2) @binding(5) var t_normal: texture_2d<f32>;
@group(2) @binding(6) var s_normal: sampler;

@group(3) @binding(0)  var t_ssao: texture_2d<f32>;
@group(3) @binding(1)  var s_ssao: sampler;
@group(3) @binding(2)  var t_bnoise: texture_2d<f32>;
@group(3) @binding(3)  var s_bnoise: sampler;
@group(3) @binding(4)  var t_sun_smap: texture_depth_2d_array;
@group(3) @binding(5)  var s_sun_smap: sampler_comparison;
@group(3) @binding(6)  var t_diffuse_irradiance: texture_cube<f32>;
@group(3) @binding(7)  var s_diffuse_irradiance: sampler;
@group(3) @binding(8)  var t_prefilter_specular: texture_cube<f32>;
@group(3) @binding(9)  var s_prefilter_specular: sampler;
@group(3) @binding(10) var t_brdf_lut: texture_2d<f32>;
@group(3) @binding(11) var s_brdf_lut: sampler;
@group(3) @binding(12) var t_lightdata: texture_2d<u32>;
@group(3) @binding(13) var s_lightdata: sampler;
@group(3) @binding(14) var t_lightdata2: texture_2d<u32>;
@group(3) @binding(15) var s_lightdata2: sampler;

#include "shadow.wgsl"
#include "pbr/render.wgsl"

const MAX_REFLECTION_LOD: f32 = 4.0;

@fragment
fn frag(@location(0) in_tint: vec4<f32>,
        @location(1) in_normal: vec3<f32>,
        @location(2) in_tangent: vec4<f32>,
        @location(3) in_wpos: vec3<f32>,
        @location(4) in_uv: vec2<f32>,
        @builtin(position) position: vec4<f32>,
        ) -> FragmentOutput {

    let albedo: vec4<f32> = textureSample(t_albedo, s_albedo, in_uv);
    var ssao = 1.0;
    if (params.ssao_enabled != 0) {
       ssao = textureSample(t_ssao, s_ssao, position.xy / params.viewport).r;
    }

    var shadow_v: f32 = 1.0;
    if (params.shadow_mapping_resolution != 0) {
        shadow_v = sampleShadow(in_wpos);
    }

    var normal = in_normal;
    if ((u_mat.flags & HAS_NORMAL_MAP) != 0u) {
        let vNt: vec3<f32> = textureSample(t_normal, s_normal, in_uv).rgb * 2.0 - 1.0;
        let vT = in_tangent.xyz;
        let sign = in_tangent.w;
        // http://www.mikktspace.com/
        let vB = sign * cross(normal, vT);
        normal = vNt.x * vT + vNt.y * vB + vNt.z * normal;
    }
    normal = normalize(normal);

    var metallic: f32 = u_mat.metallic;
    var roughness: f32 = u_mat.roughness;
    if ((u_mat.flags & HAS_METALLIC_ROUGHNESS_TEXTURE) != 0u) {
        let sampled: vec2<f32> = textureSample(t_metallic_roughness, s_metallic_rougness, in_uv).gb;
        roughness = sampled[0] * roughness;
        metallic  = sampled[1] * metallic;
    }

    let irradiance_diffuse: vec3<f32> = textureSample(t_diffuse_irradiance, s_diffuse_irradiance, normal).rgb;
    let c = mix(in_tint, vec4(1.0), metallic) * albedo;

    let V: vec3<f32> = normalize(params.cam_pos.xyz - in_wpos);
    let R: vec3<f32> = reflect(-V, normal);

    let prefilteredColor: vec3<f32> = textureSampleLevel(t_prefilter_specular, s_prefilter_specular, R, roughness * MAX_REFLECTION_LOD).rgb;

    var F0: vec3<f32> = vec3<f32>(0.04);
    F0                = mix(F0, c.rgb, vec3(metallic));

    let F_spec: vec3<f32>   = fresnelSchlickRoughness(max(dot(normal, V), 0.0), F0, roughness);
    let envBRDF: vec2<f32>  = textureSampleLevel(t_brdf_lut, s_brdf_lut, vec2(max(dot(normal, V), 0.0), roughness), 0.0).rg;
    let specular: vec3<f32> = prefilteredColor * (F_spec * envBRDF.x + envBRDF.y);

    let final_rgb: vec3<f32> = render(params.sun,
                                      V,
                                      position.xy,
                                      normal,
                                      c.rgb,
                                      F0,
                                      F_spec,
                                      params.sun_col.rgb,
                                      irradiance_diffuse,
                                      specular,
                                      metallic,
                                      roughness,
                                      shadow_v,
                                      ssao,
                                      t_lightdata,
                                      t_lightdata2,
                                      in_wpos
                                      );

    return FragmentOutput(vec4<f32>(final_rgb, c.a));
}
