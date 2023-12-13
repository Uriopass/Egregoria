use geom::{fnoise, simplex_noise, vec2, Vec2};

pub(crate) fn height(p: Vec2) -> (f32, Vec2) {
    let (noise, mut grad) = fnoise::<4>(Vec2::splat(70.69) + 0.00006 * p);
    grad *= 0.00006;

    let ratio = 0.00005;
    let mut noise = noise - 0.1 + (p.y * 2.0 - 25000.0).abs() * ratio;
    grad += vec2(0.0, (p.y * 2.0 - 25000.0).signum() * ratio);
    if noise < -0.0 {
        noise = noise * noise;
        grad = 2.0 * noise * grad;
    } else if noise > 1.0 {
        noise = 1.0;
        grad = Vec2::ZERO;
    }
    (noise, grad)
}

pub(crate) fn tree_density(mut p: Vec2) -> f32 {
    p -= vec2(-20000.0, 20000.0);
    let major = simplex_noise((p - vec2(-1000.0, 10000.0)) * 0.0006).0 * 0.5 + 0.5;
    (-major * 1.0 + simplex_noise(p * 0.0006).0 * 1.5 + 0.5).max(0.0) + -0.1
}
