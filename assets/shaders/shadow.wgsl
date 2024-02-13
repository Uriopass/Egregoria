fn sampleShadow(in_wpos: vec3<f32>) -> f32 {
    var cascade_idx = 100;
    var blend = 0.0;
    for (var i = 0 ; i < N_SHADOWS ; i++) {
        let light_local: vec4<f32> = params.sunproj[i] * vec4(in_wpos, 1.0);
        let corrected: vec3<f32> = light_local.xyz / light_local.w * vec3(0.5, -0.5, 1.0) + vec3(0.5, 0.5, 0.0);

        if (corrected.z >= 0.1 && corrected.z <= 1.0 && corrected.x >= 0.0 && corrected.x <= 1.0 && corrected.y >= 0.0 && corrected.y <= 1.0) {
            cascade_idx = i;
            blend = smoothstep(0.5, 1.0, 2.0 * length(corrected.xy - vec2(0.5, 0.5)));
            break;
        }
    }
    if (cascade_idx == 100) {
        return 1.0;
    }

    var s1 = 1.0;
    var s2 = 1.0;
    if (blend < 1.0 - 1e-3) {
        s1 = sampleOneShadow(in_wpos, cascade_idx);
        if (cascade_idx == N_SHADOWS - 1) {
            return s1;
        }
    }
    if (blend > 1e-3) {
        s2 = sampleOneShadow(in_wpos, (cascade_idx + 1)%4);
    }
    return mix(s1, s2, blend);
}

fn sampleFirstShadow(in_wpos: vec3<f32>) -> f32 {
    return sampleOneShadow(in_wpos, 0);
}

fn sampleOneShadow(in_wpos: vec3<f32>, index: i32) -> f32 {
    let light_local: vec4<f32> = params.sunproj[index] * vec4(in_wpos, 1.0);

    let corrected: vec3<f32> = light_local.xyz / light_local.w * vec3(0.5, -0.5, 1.0) + vec3(0.5, 0.5, 0.0);

    var total: f32 = 0.0;
    let offset: f32 = 1.0 / f32(params.shadow_mapping_resolution);

    var x: i32;

    for (var y = -1 ; y <= 1 ; y++) {
        x = -1;
        for (; x <= 1; x++) {
            let shadow_coord: vec3<f32> = corrected + vec3(f32(x), f32(y), 0.0) * offset;
            total += textureSampleCompare(t_sun_smap, s_sun_smap, shadow_coord.xy, index, shadow_coord.z);
        }
    }
    if (corrected.z < 0.0 || corrected.z > 1.0 || corrected.x < 0.0 || corrected.x > 1.0 || corrected.y < 0.0 || corrected.y > 1.0) {
        return 1.0;
    }

    return total / 9.0;
}