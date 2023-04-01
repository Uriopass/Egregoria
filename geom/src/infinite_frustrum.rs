use crate::{Intersect3, Matrix4, Plane, Shape3, Vec3, Vec4, AABB3};

/// A [`Frustrum'] with infinite far plane.
/// The planes must be in the following order:
pub struct InfiniteFrustrum {
    /// [near, left, right, bottom, top]
    planes: [Plane; 5],
}

impl InfiniteFrustrum {
    /// Create a new frustrum from the given planes.
    /// The planes must be in the following order:
    /// [near, left, right, bottom, top, far]
    /// where the normals are pointing inwards.
    pub fn new(planes: [Plane; 5]) -> Self {
        Self { planes }
    }

    /// Create a new frustrum from a reversed z perspective inverse view-projection matrix.
    #[rustfmt::skip]
    pub fn from_reversez_invviewproj(eye: Vec3, inv_viewproj: Matrix4) -> Self {
        let nlb = (inv_viewproj * Vec4::new(-1.0, -1.0, 1.0, 1.0)).xyz();
        let nlt = (inv_viewproj * Vec4::new(-1.0,  1.0, 1.0, 1.0)).xyz();
        let nrb = (inv_viewproj * Vec4::new( 1.0, -1.0, 1.0, 1.0)).xyz();
        let nrt = (inv_viewproj * Vec4::new( 1.0,  1.0, 1.0, 1.0)).xyz();
        let flt = (nlt - eye) * 1000.0 + eye;
        let frb = (nrb - eye) * 1000.0 + eye;

        let near = Plane::from_points(nlb, nlt, nrb);
        let left = Plane::from_points(nlt, nlb, flt);
        let right = Plane::from_points(nrb, nrt, frb);
        let bottom = Plane::from_points(nrb, frb, nlb);
        let top = Plane::from_points(nlt, flt, nrt);

        Self::new([near, left, right, bottom, top])
    }
}

impl Shape3 for InfiniteFrustrum {
    fn bbox(&self) -> AABB3 {
        unimplemented!("InfiniteFrustrum does not have a finite bounding box")
    }
}

defer_inter3!(Vec3 => InfiniteFrustrum);
impl Intersect3<Vec3> for InfiniteFrustrum {
    fn intersects(&self, v: &Vec3) -> bool {
        for plane in &self.planes {
            if !plane.point_is_positive(*v) {
                return false;
            }
        }
        true
    }
}

defer_inter3!(AABB3 => InfiniteFrustrum);
impl Intersect3<AABB3> for InfiniteFrustrum {
    fn intersects(&self, aabb: &AABB3) -> bool {
        for plane in &self.planes {
            let mut out = false;
            out |= plane.point_is_positive(aabb.ll);
            out |= plane.point_is_positive(Vec3::new(aabb.ur.x, aabb.ll.y, aabb.ll.z));
            out |= plane.point_is_positive(Vec3::new(aabb.ll.x, aabb.ur.y, aabb.ll.z));
            out |= plane.point_is_positive(Vec3::new(aabb.ur.x, aabb.ur.y, aabb.ll.z));
            out |= plane.point_is_positive(Vec3::new(aabb.ll.x, aabb.ll.y, aabb.ur.z));
            out |= plane.point_is_positive(Vec3::new(aabb.ur.x, aabb.ll.y, aabb.ur.z));
            out |= plane.point_is_positive(Vec3::new(aabb.ll.x, aabb.ur.y, aabb.ur.z));
            out |= plane.point_is_positive(aabb.ur);
            if !out {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Plane;

    #[test]
    fn test_frustrum() {
        let planes = [
            Plane::new(Vec3::new(0.0, 0.0, 1.0), 0.0),
            Plane::new(Vec3::new(1.0, 0.0, 0.0), 0.0),
            Plane::new(Vec3::new(-1.0, 0.0, 0.0), -1.0),
            Plane::new(Vec3::new(0.0, 1.0, 0.0), 0.0),
            Plane::new(Vec3::new(0.0, -1.0, 0.0), -1.0),
        ];

        let f = InfiniteFrustrum::new(planes);

        assert!(f.intersects(&Vec3::new(0.0, 0.0, 0.0)));
        assert!(f.intersects(&Vec3::new(0.0, 0.0, 1.0)));
        assert!(!f.intersects(&Vec3::new(0.0, 0.0, -1.0)));
        assert!(f.intersects(&Vec3::new(0.0, 1.0, 0.0)));
        assert!(!f.intersects(&Vec3::new(0.0, -1.0, 0.0)));
        assert!(f.intersects(&Vec3::new(1.0, 0.0, 0.0)));
        assert!(!f.intersects(&Vec3::new(-1.0, 0.0, 0.0)));

        let aabb = AABB3::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(f.intersects(&aabb));
        let aabb = AABB3::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(0.0, 0.0, 0.0));
        assert!(f.intersects(&aabb));
        let aabb = AABB3::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(f.intersects(&aabb));
        let aabb = AABB3::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        assert!(f.intersects(&aabb));
        let aabb = AABB3::new(Vec3::new(-2.0, -2.0, -2.0), Vec3::new(-1.0, -1.0, -1.0));
        assert!(!f.intersects(&aabb));
    }
}