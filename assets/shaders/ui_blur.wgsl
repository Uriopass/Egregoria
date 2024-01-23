struct VertexOutput {
    @location(0) v_TexCoord: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@vertex
fn vert(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var tc: vec2<f32> = vec2(0.0, 0.0);
    switch (vi) {
        case 0u: {tc = vec2(0.0, 0.0);}
        case 1u: {tc = vec2(2.0, 0.0);}
        case 2u: {tc = vec2(0.0, 2.0);}
        default: {}
    }
    let pos: vec2<f32> = tc * 2.0 - 1.0;
    let gl_Position = vec4(pos.x, -pos.y, 0.5, 1.0);

    return VertexOutput(tc, gl_Position);
}

struct FragmentOutput {
    @location(0) o_Target: vec4<f32>,
}

@group(0) @binding(0) var t_Color: texture_2d<f32>;
@group(0) @binding(1) var s_Color: sampler;

@fragment
fn downscale(@location(0) v_TexCoord: vec2<f32>) -> FragmentOutput {
    let dim: vec2<u32> = textureDimensions(t_Color, 0);
    let halfpixel = vec2<f32>(0.5 / f32(dim.x), 0.5 / f32(dim.y));
    let halfpixel_rot = vec2<f32>(halfpixel.x, -halfpixel.y);

    var sum: vec3<f32> = textureSampleLevel(t_Color, s_Color, v_TexCoord, 0.0).rgb * 4.0;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + halfpixel, 0.0).rgb;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord - halfpixel, 0.0).rgb;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + halfpixel_rot, 0.0).rgb;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord - halfpixel_rot, 0.0).rgb;
    return FragmentOutput(vec4(sum / 8.0, 1.0));
}

@fragment
fn upscale(@location(0) v_TexCoord: vec2<f32>) -> FragmentOutput {
    let dim: vec2<u32> = textureDimensions(t_Color, 0);
    let halfpixel = vec2<f32>(0.5 / f32(dim.x), 0.5 / f32(dim.y));

    var sum: vec3<f32> = textureSampleLevel(t_Color, s_Color, v_TexCoord + vec2(-halfpixel.x * 2.0, 0.0), 0.0).rgb;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + vec2(-halfpixel.x, halfpixel.y), 0.0).rgb * 2.0;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + vec2(0.0, halfpixel.y * 2.0), 0.0).rgb;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + vec2(halfpixel.x, halfpixel.y), 0.0).rgb * 2.0;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + vec2(halfpixel.x * 2.0, 0.0), 0.0).rgb;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + vec2(halfpixel.x, -halfpixel.y), 0.0).rgb * 2.0;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + vec2(0.0, -halfpixel.y * 2.0), 0.0).rgb;
    sum += textureSampleLevel(t_Color, s_Color, v_TexCoord + vec2(-halfpixel.x, -halfpixel.y), 0.0).rgb * 2.0;
    return FragmentOutput(vec4(sum / 12.0, 1.0));
}
