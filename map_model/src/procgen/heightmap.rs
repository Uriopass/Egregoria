use geom::{vec2, vec3, Vec2, Vec3};

fn permute(x: Vec3) -> Vec3 {
    Vec3::modulo(((x * 34.0) + Vec3::splat(1.0)) * x, 289.0)
}

const CX: Vec2 = Vec2::new(0.211_324_87, 0.211_324_87);
const CY: Vec2 = Vec2::new(0.366_025_42, 0.366_025_42);
const CZ: Vec2 = Vec2::new(-0.577_350_26, -0.577_350_26);
const CW: Vec3 = Vec3::new(0.024_390_243, 0.024_390_243, 0.024_390_243);

#[allow(clippy::many_single_char_names)]
pub fn simplex_noise(pos: Vec2) -> f32 {
    let mut i: Vec2 = Vec2::floor(pos + Vec2::splat(Vec2::dot(pos, CY)));
    let x0: Vec2 = pos - i + Vec2::splat(Vec2::dot(i, CX));
    let i1 = if x0.x > x0.y {
        vec2(1.0, 0.0)
    } else {
        vec2(0.0, 1.0)
    };
    let x12_lo = x0 + CX - i1;
    let x12_hi = x0 + CZ;
    i.x %= 289.0;
    i.y %= 289.0;
    let p: Vec3 = permute(
        permute(Vec3::splat(i.y) + vec3(0.0, i1.y, 1.0)) + Vec3::splat(i.x) + vec3(0.0, i1.x, 1.0),
    );
    let mut m: Vec3 = (Vec3::splat(0.5)
        - vec3(x0.magnitude2(), x12_lo.magnitude2(), x12_hi.magnitude2()))
    .max(Vec3::ZERO);
    m = m * m;
    m = m * m;
    let x = 2.0 * (p * CW).fract() - Vec3::splat(1.0);
    let h = x.abs() - Vec3::splat(0.5);
    let ox = Vec3::floor(x + Vec3::splat(0.5));
    let a0 = x - ox;
    m *= Vec3::splat(1.792_842_9) - Vec3::splat(0.853_734_73) * (a0 * a0 + h * h);
    let g: Vec3 = Vec3 {
        x: a0.x * x0.x + h.x * x0.y,
        y: a0.y * x12_lo.x + h.y * x12_lo.y,
        z: a0.z * x12_hi.x + h.z * x12_hi.y,
    };
    130.0 * m.dot(g)
}

const FBM_MAG: f32 = 0.4;

fn fnoise(ampl: f32, in_wv: Vec2) -> f32 {
    let mut dec: Vec2 = Vec2::splat(1.0) + in_wv * ampl;

    let mut noise: f32 = 0.0;
    let mut amplitude: f32 = 0.6;

    for _ in 0..8 {
        noise += amplitude * (simplex_noise(dec) + 1.0) * FBM_MAG;
        dec = dec * (1.0 / FBM_MAG);
        amplitude *= FBM_MAG;
    }

    noise
}

pub fn height(mut p: Vec2) -> f32 {
    p -= vec2(4000.0, 4000.0);

    let mut noise = fnoise(0.00007, p);

    noise -= p.magnitude() * 0.00009;
    noise.max(0.0)
}
