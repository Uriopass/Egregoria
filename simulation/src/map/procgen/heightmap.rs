use geom::{vec2, vec3, Vec2, Vec3};

fn permute(x: f32) -> f32 {
    ((x * 34.0 + 1.0) * x) % 289.0
}

const CX: Vec2 = Vec2::new(0.211_324_87, 0.211_324_87);
const CY: Vec2 = Vec2::new(0.366_025_42, 0.366_025_42);
const CZ: Vec2 = Vec2::new(-0.577_350_26, -0.577_350_26);
const K: f32 = 0.024_390_243;

// Gradient mapping with an extra rotation.
fn grad2(p: Vec2) -> Vec2 {
    // Map from a line to a diamond such that a shift maps to a rotation.
    let u = permute(permute(p.x) + p.y) * K;
    let u = 4.0 * u.fract() - 2.0;
    vec2(u.abs() - 1.0, ((u + 1.0).abs() - 2.0).abs() - 1.0)
}

/* return range is [-0.5; 0.5] */
#[allow(clippy::many_single_char_names)]
#[inline(always)]
pub(crate) fn simplex_noise(pos: Vec2) -> (f32, Vec2) {
    let mut i: Vec2 = Vec2::floor(pos + Vec2::splat(Vec2::dot(pos, CY)));
    let x0: Vec2 = pos - i + Vec2::splat(Vec2::dot(i, CX));
    let i1 = if x0.x > x0.y {
        vec2(1.0, 0.0)
    } else {
        vec2(0.0, 1.0)
    };
    let v1 = x0 + CX - i1;
    let v2 = x0 + CZ;

    i.x %= 289.0;
    i.y %= 289.0;

    let t: Vec3 = (Vec3::splat(0.5) - vec3(x0.mag2(), v1.mag2(), v2.mag2())).max(Vec3::ZERO);
    let t2: Vec3 = t * t;
    let t4 = t2 * t2;

    let g0 = grad2(i);
    let g1 = grad2(i + i1);
    let g2 = grad2(i + Vec2::splat(1.0));

    let gv = vec3(g0.dot(x0), g1.dot(v1), g2.dot(v2));

    // Compute partial derivatives in x and y
    let temp = t2 * t * gv;
    let mut grad = -8.0
        * vec2(
            temp.dot(vec3(x0.x, v1.x, v2.x)),
            temp.dot(vec3(x0.y, v1.y, v2.y)),
        );
    grad.x += t4.dot(vec3(g0.x, g1.x, g2.x));
    grad.y += t4.dot(vec3(g0.y, g1.y, g2.y));
    grad = 40.0 * grad;

    (40.0 * t4.dot(gv), grad)
}

const FBM_MAG: f32 = 0.4;

fn fnoise(ampl: f32, in_wv: Vec2) -> (f32, Vec2) {
    let mut dec = Vec2::splat(70.69) + in_wv * ampl;

    let mut noise: f32 = 0.0;
    let mut amplitude: f32 = 1.0;
    let mut grad: Vec2 = Vec2::ZERO;

    for _ in 0..4 {
        let (n, g) = simplex_noise(dec);
        noise += amplitude * n;
        grad += g;

        dec *= 1.0 / FBM_MAG;
        amplitude *= FBM_MAG;
    }

    (noise, grad * ampl)
}

pub(crate) fn height(p: Vec2) -> (f32, Vec2) {
    //p -= vec2(-2000.0, 2000.0);

    let (noise, mut grad) = fnoise(0.00003, p);
    let ratio = 0.00005;
    let mut noise = noise - 0.1 + (p.y - 25000.0).abs() * ratio;
    grad += vec2(0.0, (p.y - 25000.0).signum() * ratio);
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
    let major = simplex_noise((p - vec2(-1000.0, 10000.0)) * 0.0003).0 * 0.5 + 0.5;
    (-major * 1.0 + simplex_noise(p * 0.0003).0 * 1.5 + 0.5).max(0.0) + -0.1
}
