fn dither(frag: vec2<f32>) -> f32 {
    let color: f32 = textureSample(t_bnoise, s_bnoise, frag / 512.0).r;
    return (color - 0.5) / 255.0;
}