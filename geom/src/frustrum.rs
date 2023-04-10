use crate::{minmax3, Intersect3, Plane, Shape3, Vec3, AABB3};

pub struct Frustrum {
    /// [near, left, right, bottom, top, far]
    pub planes: [Plane; 6],
    /// Plane intersection points.
    ///
    /// - n: near, f: far
    /// - l: left, r: right
    /// - t: top,  b: bottom
    ///
    /// all pointing inwards
    ///
    /// [nlt, nrt, nlb, nrb, flt, frt, flb, frb]
    pub points: [Vec3; 8],
    /// Bounding box of the frustrum.
    pub aabb: AABB3,
}

impl Frustrum {
    /// Create a new frustrum from the given planes.
    /// The planes must be in the following order:
    /// [near, left, right, bottom, top, far]
    /// where the normals are pointing inwards.
    pub fn new(planes: [Plane; 6]) -> Self {
        let near = &planes[0];
        let left = &planes[1];
        let right = &planes[2];
        let bottom = &planes[3];
        let top = &planes[4];
        let far = &planes[5];

        let nlt = intersect_planes(near, left, top);
        let nrt = intersect_planes(near, right, top);
        let nlb = intersect_planes(near, left, bottom);
        let nrb = intersect_planes(near, right, bottom);
        let flt = intersect_planes(far, left, top);
        let frt = intersect_planes(far, right, top);
        let flb = intersect_planes(far, left, bottom);
        let frb = intersect_planes(far, right, bottom);

        let points = [nlt, nrt, nlb, nrb, flt, frt, flb, frb];

        let (minn, maxx) = minmax3(points).unwrap();
        let aabb = AABB3::new(minn, maxx);

        Self {
            planes,
            points: [nlt, nrt, nlb, nrb, flt, frt, flb, frb],
            aabb,
        }
    }
}

fn intersect_planes(p0: &Plane, p1: &Plane, p2: &Plane) -> Vec3 {
    let n0 = p0.n;
    let n1 = p1.n;
    let n2 = p2.n;
    let o0 = p0.o;
    let o1 = p1.o;
    let o2 = p2.o;

    let n0n1 = n0.cross(n1);
    let n1n2 = n1.cross(n2);
    let n2n0 = n2.cross(n0);

    let n0n1n2 = n0n1.dot(n2);

    let p = n0n1 * o2 + n1n2 * o0 + n2n0 * o1;
    p / n0n1n2
}

impl Shape3 for Frustrum {
    fn bbox(&self) -> AABB3 {
        self.aabb
    }
}

defer_inter3!(Vec3 => Frustrum);
impl Intersect3<Vec3> for Frustrum {
    fn intersects(&self, v: &Vec3) -> bool {
        for plane in &self.planes {
            if !plane.point_is_positive(*v) {
                return false;
            }
        }
        true
    }
}

defer_inter3!(AABB3 => Frustrum);
impl Intersect3<AABB3> for Frustrum {
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

        let mut out = true;
        let mut out1 = true;
        let mut out2 = true;
        let mut out3 = true;
        let mut out4 = true;
        let mut out5 = true;
        for point in &self.points {
            out1 &= point.x < aabb.ll.x;
            out3 &= point.y < aabb.ll.y;
            out5 &= point.z < aabb.ll.z;
            out2 &= point.y > aabb.ur.y;
            out4 &= point.z > aabb.ur.z;
            out &= point.x > aabb.ur.x;
        }
        if out | out1 | out2 | out3 | out4 | out5 {
            return false;
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
            Plane::new(Vec3::new(0.0, 0.0, -1.0), -1.0),
        ];

        let f = Frustrum::new(planes);

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

    #[test]
    fn test_intersect_planes() {
        let v = intersect_planes(
            &Plane::new(Vec3::new(1.0, 0.0, 0.0), 1.0),
            &Plane::new(Vec3::new(0.0, 1.0, 0.0), 1.0),
            &Plane::new(Vec3::new(0.0, 0.0, 1.0), 1.0),
        );
        assert_eq!(v, Vec3::new(1.0, 1.0, 1.0));
    }
}
