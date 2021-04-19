use crate::Vec3;

pub struct Plane {
    pub p: Vec3,
    pub n: Vec3,
}

impl Plane {
    pub fn new(p: Vec3, n: Vec3) -> Self {
        Plane { p, n }
    }
}
