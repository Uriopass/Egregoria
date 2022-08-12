struct VertexOutput {
    @location(0) v_TexCoord: vec2<f32>,
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var tc: vec2<f32> = vec2(0.0, 0.0);
    switch (vi) {
        case 0u: {tc = vec2(1.0, 0.0);}
        case 1u: {tc = vec2(1.0, 1.0);}
        case 2u: {tc = vec2(0.0, 0.0);}
        case 3u: {tc = vec2(0.0, 1.0);}
        default: {}
    }
    let pos: vec2<f32> = tc * 2.0 - 1.0;
    let gl_Position = vec4(pos.x, -pos.y, 0.5, 1.0);

    return VertexOutput(tc, gl_Position);
}
