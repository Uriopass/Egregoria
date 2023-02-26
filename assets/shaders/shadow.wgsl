fn sampleShadow(in_wpos: vec3<f32>) -> f32 {
    let light_local: vec4<f32> = params.sunproj * vec4(in_wpos, 1.0);

    let corrected: vec3<f32> = light_local.xyz / light_local.w * vec3(0.5, -0.5, 1.0) + vec3(0.5, 0.5, 0.0);

    var total: f32 = 0.0;
    let offset: f32 = 1.0 / f32(params.shadow_mapping_resolution);

    var x: i32;

    for (var y = -1 ; y <= 1 ; y++) {
        x = -1;
        for (; x <= 1; x++) {
            let shadow_coord: vec3<f32> = corrected + vec3(f32(x), f32(y), -1.0) * offset;
            total += textureSampleCompare(t_sun_smap, s_sun_smap, shadow_coord.xy, shadow_coord.z);
        }
    }

    total = total / 9.0;

    if (light_local.z >= 1.0) {
        return 1.0;
    }
    return mix(total, 1.0, clamp(dot(light_local.xy, light_local.xy), 0.0, 1.0));
}