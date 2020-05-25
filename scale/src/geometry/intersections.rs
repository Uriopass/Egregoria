use super::Vec2;

#[derive(Clone, Copy)]
pub struct Ray {
    pub from: Vec2,
    pub dir: Vec2,
}

pub fn intersection_point(r1: Ray, r2: Ray) -> Option<Vec2> {
    let div = r1.dir.perp_dot(r2.dir);

    let p_diff = r1.from - r2.from;
    let t = -r2.dir.perp_dot(p_diff);
    let s = -r1.dir.perp_dot(p_diff);

    if t * div > 0.0 && s * div > 0.0 {
        Some(r1.from + r1.dir * t / div)
    } else {
        None
    }
}

pub fn both_dist_to_inter(r1: Ray, r2: Ray) -> Option<(f32, f32)> {
    let div = r1.dir.perp_dot(r2.dir);

    let p_diff = r1.from - r2.from;
    let t = -r2.dir.perp_dot(p_diff);
    let s = -r1.dir.perp_dot(p_diff);

    if t * div > 0.0 && s * div > 0.0 {
        Some((t / div, s / div))
    } else {
        None
    }
}

pub fn time_to_hit(dist: f32, v0: f32, acc: f32) -> f32 {
    // acc * t² / 2.0 + t*v0 - dist = 0
    // delta = v0² + 2 * acc * dist
    // t = (-v0 + sqrt(v0² + 2*acc*dist)) / acc
    (-v0 + (v0 * v0 + 2.0 * acc * dist).sqrt()) / acc
}
