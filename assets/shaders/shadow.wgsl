#include "shadow_exp.wgsl"

fn sampleShadow(
    in_wpos: vec3<f32>,
    shadow_mapping_resolution: i32,
    t_sun_smap: texture_depth_2d_array,
    s_sun_smap: sampler,
) -> f32 {
    var cascade_idx = 0;
    var i: i32 = 0;
    for (; i < 3 ; i++) {
        let light_local: vec4<f32> = params.sunproj[i] * vec4(in_wpos, 1.0);
        let corrected: vec3<f32> = light_local.xyz / light_local.w * vec3(0.5, -0.5, 1.0) + vec3(0.5, 0.5, 0.0);

        if (corrected.z >= 0.1 && corrected.z <= 1.0 && corrected.x >= 0.0 && corrected.x <= 1.0 && corrected.y >= 0.0 && corrected.y <= 1.0) {
            cascade_idx = i;
            break;
        }
    }

    return sampleOneShadow(in_wpos, cascade_idx, params.sunproj[cascade_idx], t_sun_smap, s_sun_smap);
}

fn sampleOneShadow(in_wpos: vec3<f32>, cascade: i32, sunproj: mat4x4<f32>, t_sun_smap: texture_depth_2d_array, s_sun_smap: sampler) -> f32 {
    let light_local: vec4<f32> = sunproj * vec4(in_wpos, 1.0);

    let corrected: vec3<f32> = light_local.xyz / light_local.w * vec3(0.5, -0.5, 1.0) + vec3(0.5, 0.5, 0.0);

    let z: f32 = textureSample(t_sun_smap, s_sun_smap, corrected.xy, cascade);
    let d: f32 = corrected.z;

    var v: f32 = clamp(z * exp(-EXP_C * d + EXP_C), 0.0, 1.0);

    return v;
   /*
    var total: f32 = 0.0;
    let offset: f32 = 1.0 / f32(params.shadow_mapping_resolution);

    var x: i32;

    for (var y = -1 ; y <= 1 ; y++) {
        x = -1;
        for (; x <= 1; x++) {
            let shadow_coord: vec3<f32> = corrected + vec3(f32(x), f32(y), -1.0) * offset;
            total += textureSampleCompare(t_sun_smap, s_sun_smap, shadow_coord.xy, index, shadow_coord.z);
        }
    }
    if (corrected.z < 0.0 || corrected.z > 1.0 || corrected.x < 0.0 || corrected.x > 1.0 || corrected.y < 0.0 || corrected.y > 1.0) {
        return 1.0;
    }

    return total / 9.0;*/
}