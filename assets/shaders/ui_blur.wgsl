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
    let o_Target = textureSampleLevel(t_Color, s_Color, v_TexCoord, 0.0);
    return FragmentOutput(o_Target);
}

@fragment
fn upscale(@location(0) v_TexCoord: vec2<f32>) -> FragmentOutput {
    let o_Target = textureSampleLevel(t_Color, s_Color, v_TexCoord, 0.0);
    return FragmentOutput(o_Target);
}
