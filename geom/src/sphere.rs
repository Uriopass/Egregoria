use crate::Vec3;

#[derive(Default, Copy, Clone, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    #[inline]
    pub const fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 0.0,
        }
    }
}
