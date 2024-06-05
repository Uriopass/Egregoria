use crate::Vec3;

/// Plane is defined by a normal vector and an offset
/// where the offset is the distance from the origin of any point on the plane projected on the normal vector.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Plane {
    /// Normal vector
    pub n: Vec3,
    /// Offset
    pub o: f32,
}

impl Plane {
    pub const X: Self = Self { n: Vec3::X, o: 0.0 };

    #[inline]
    pub const fn new(n: Vec3, o: f32) -> Self {
        Self { n, o }
    }

    pub fn point_is_positive(&self, p: Vec3) -> bool {
        self.n.dot(p) - self.o >= 0.0
    }

    pub(crate) fn from_points(p0: Vec3, p1: Vec3, p2: Vec3) -> Self {
        let n = (p1 - p0).cross(p2 - p0).normalize();
        let o = n.dot(p0);
        Self { n, o }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane() {
        let p = Plane::new(Vec3::new(0.0, 0.0, 1.0), 1.0);
        assert!(p.point_is_positive(Vec3::new(0.0, 0.0, 1.0)));
        assert!(p.point_is_positive(Vec3::new(0.0, 0.0, 2.0)));
        assert!(!p.point_is_positive(Vec3::new(0.0, 0.0, 0.0)));
        assert!(!p.point_is_positive(Vec3::new(0.0, 0.0, -1.0)));
    }
}
