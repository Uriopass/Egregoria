use crate::aabb::AABB;
use crate::{vec2, Circle, Intersect, Polygon, Segment, Shape, Vec2};
use serde::{Deserialize, Serialize};
use std::hint::unreachable_unchecked;
use std::ops::Add;

/// Oriented bounding box
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OBB {
    pub corners: [Vec2; 4],
}

impl OBB {
    pub const ZERO: Self = Self {
        corners: [Vec2::ZERO; 4],
    };

    /// cossin of `UNIT_X` makes this an AABB
    #[inline]
    pub fn new(center: Vec2, cossin: Vec2, w: f32, h: f32) -> Self {
        let up = cossin * w * 0.5;
        let right = cossin.perpendicular() * h * 0.5;
        Self {
            corners: [
                center - up - right,
                center - up + right,
                center + up + right,
                center + up - right,
            ],
        }
    }

    #[inline]
    pub fn new_corners(corners: [Vec2; 4]) -> Self {
        Self { corners }
    }

    #[inline]
    pub fn from_rotated_aabb(aabb: AABB, cossin: Vec2) -> Self {
        Self {
            corners: [
                aabb.ll.rotated_by(cossin),
                vec2(aabb.ur.x, aabb.ll.y).rotated_by(cossin),
                aabb.ur.rotated_by(cossin),
                vec2(aabb.ll.x, aabb.ur.y).rotated_by(cossin),
            ],
        }
    }

    #[inline]
    pub fn axis(&self) -> [Vec2; 2] {
        [
            self.corners[1] - self.corners[0],
            self.corners[3] - self.corners[0],
        ]
    }

    #[inline]
    /// Calculates the moment of inertia of the OBB around its center.
    pub fn moment_of_inertia(&self, mass: f32) -> f32 {
        let [a, b] = self.axis();
        let a = a.mag2();
        let b = b.mag2();
        mass * (a + b) / 12.0
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.corners[2] + self.corners[0]) * 0.5
    }

    #[inline]
    pub fn expand(&self, w: f32) -> Self {
        let [a, b] = self.axis();
        let a = match a.try_normalize() {
            Some(x) => x,
            None => return *self,
        };
        let b = match b.try_normalize() {
            Some(x) => x,
            None => return *self,
        };
        Self {
            corners: [
                self.corners[0] - a * w - b * w,
                self.corners[1] + a * w - b * w,
                self.corners[2] + a * w + b * w,
                self.corners[3] - a * w + b * w,
            ],
        }
    }

    pub fn map(&self, f: impl Fn(Vec2) -> Vec2) -> Self {
        Self {
            corners: [
                f(self.corners[0]),
                f(self.corners[1]),
                f(self.corners[2]),
                f(self.corners[3]),
            ],
        }
    }

    /// Returns true if other overlaps one dimension of this.
    /// Taken from <https://www.flipcode.com/archives/2D_OBB_Intersection.shtml>
    fn intersects1way(&self, other: &OBB) -> bool {
        let mut axis = self.axis();

        // Make the length of each axis 1/edge length so we know any
        // dot product must be less than 1 to fall within the edge.
        axis[0] /= axis[0].mag2();
        axis[1] /= axis[1].mag2();

        for &axis in &axis {
            let origin = self.corners[0].dot(axis);

            // Find the extent of box 2 on axis a
            let mut t_min = other.corners[0].dot(axis);
            let mut t_max = t_min;

            let ts = [
                other.corners[1].dot(axis),
                other.corners[2].dot(axis),
                other.corners[3].dot(axis),
            ];

            for &t in &ts {
                t_min = t_min.min(t);
                t_max = t_max.max(t);
            }

            // We have to subtract off the origin

            // See if [t_min, t_max] intersects [0, 1]
            if (t_min > 1.0 + origin) || (t_max < origin) {
                // There was no intersection along this dimension;
                // the boxes cannot possibly overlap.
                return false;
            }
        }

        // There was no dimension along which there is no intersection.
        // Therefore the boxes overlap.
        true
    }

    #[inline]
    pub fn contains(&self, p: Vec2) -> bool {
        let ok0 = (self.corners[1] - self.corners[0]).dot(p - self.corners[0]) > 0.0;
        let ok1 = (self.corners[2] - self.corners[1]).dot(p - self.corners[1]) > 0.0;
        let ok2 = (self.corners[3] - self.corners[2]).dot(p - self.corners[2]) > 0.0;
        let ok3 = (self.corners[0] - self.corners[3]).dot(p - self.corners[3]) > 0.0;
        ok0 & ok1 & ok2 & ok3
    }

    #[inline]
    pub fn is_close(&self, p: Vec2, dist: f32) -> bool {
        if self.contains(p) {
            return true;
        }
        let d = Segment {
            src: self.corners[3],
            dst: self.corners[0],
        }
        .project(p)
        .is_close(p, dist);

        if d {
            return true;
        }
        for i in 0..3 {
            let d = Segment {
                src: self.corners[i],
                dst: self.corners[i + 1],
            }
            .project(p)
            .is_close(p, dist);
            if d {
                return true;
            }
        }
        false
    }

    /// Returns [bottom, right, up, left]
    ///      3
    ///   +--<--+
    /// 4 v     ^ 2
    ///   +-->--+
    ///      1
    #[inline]
    pub fn segments(&self) -> [Segment; 4] {
        [
            Segment::new(self.corners[0], self.corners[1]),
            Segment::new(self.corners[1], self.corners[2]),
            Segment::new(self.corners[2], self.corners[3]),
            Segment::new(self.corners[3], self.corners[0]),
        ]
    }
}

impl Add<Vec2> for OBB {
    type Output = OBB;

    #[inline]
    fn add(self, rhs: Vec2) -> Self::Output {
        Self {
            corners: [
                self.corners[0] + rhs,
                self.corners[1] + rhs,
                self.corners[2] + rhs,
                self.corners[3] + rhs,
            ],
        }
    }
}

impl Add<OBB> for Vec2 {
    type Output = OBB;

    #[inline]
    fn add(self, rhs: OBB) -> Self::Output {
        rhs + self
    }
}

impl Shape for OBB {
    #[inline]
    fn bbox(&self) -> AABB {
        let (min, max) = match super::minmax(self.corners.iter().copied()) {
            Some(x) => x,
            None => unsafe { unreachable_unchecked() },
        };

        AABB::new_ll_ur(min, max)
    }
}

impl Intersect<Vec2> for OBB {
    #[inline]
    fn intersects(&self, &p: &Vec2) -> bool {
        self.contains(p)
    }
}

impl Intersect<OBB> for OBB {
    #[inline]
    fn intersects(&self, other: &OBB) -> bool {
        self.intersects1way(other) && other.intersects1way(self)
    }
}

defer_inter!(OBB => Polygon);

impl Intersect<Circle> for OBB {
    #[inline]
    fn intersects(&self, shape: &Circle) -> bool {
        self.contains(shape.center)
            || self
                .segments()
                .iter()
                .any(|s| s.project(shape.center).is_close(shape.center, shape.radius))
    }
}

impl Intersect<AABB> for OBB {
    #[inline]
    fn intersects(&self, shape: &AABB) -> bool {
        let Vec2 {
            x: mut min_x,
            y: mut min_y,
        } = self.corners[0];
        let mut max_x = min_x;
        let mut max_y = min_y;

        for c in &self.corners[1..4] {
            min_x = min_x.min(c.x);
            max_x = max_x.max(c.x);
            min_y = min_y.min(c.y);
            max_y = max_y.max(c.y);
        }

        let v =
            min_x > shape.ur.x || max_x < shape.ll.x || min_y > shape.ur.y || max_y < shape.ll.y;

        v || self.intersects1way(&shape.into())
    }
}

impl From<&AABB> for OBB {
    #[inline]
    fn from(aabb: &AABB) -> Self {
        Self {
            corners: [
                aabb.ll,
                vec2(aabb.ur.x, aabb.ll.y),
                aabb.ur,
                vec2(aabb.ll.x, aabb.ur.y),
            ],
        }
    }
}

defer_inter!(Segment => OBB);
impl Intersect<Segment> for OBB {
    fn intersects(&self, shape: &Segment) -> bool {
        let axis = self.axis();
        let w = axis[0].mag();
        let h = axis[1].mag();
        let tr = Segment {
            src: (shape.src - self.corners[0]).rotated_by(axis[0].flipy()),
            dst: (shape.dst - self.corners[0]).rotated_by(axis[0].flipy()),
        };
        AABB::new_ll_ur(Vec2::ZERO, vec2(w * w, h * w)).intersects(&tr)
    }
}

#[cfg(test)]
mod tests {
    use crate::{vec2, Intersect, Segment, Shape, Vec2, OBB};

    #[test]
    fn test_segobb() {
        let mut obb = OBB {
            corners: [Vec2::ZERO, vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0)],
        };
        for lol in &mut obb.corners {
            *lol -= Vec2::splat(0.5);
            *lol = lol.rotated_by(vec2(
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ))
        }
        assert!(!obb.intersects(&Segment::new(vec2(-0.71, 0.0), vec2(0.0, -0.71))));
        assert!(obb.intersects(&Segment::new(vec2(-0.70, 0.0), vec2(0.0, -0.70))));
    }

    #[test]
    fn test_obbobb() {
        let obb = OBB {
            corners: [
                Vec2::ZERO,
                vec2(10.0, 0.0),
                vec2(10.0, 10.0),
                vec2(0.0, 10.0),
            ],
        };

        let obb_contained = OBB {
            corners: [
                vec2(1.0, 1.0),
                vec2(2.0, 1.0),
                vec2(2.0, 2.0),
                vec2(1.0, 2.0),
            ],
        };

        assert!(obb.intersects(&obb_contained));
        assert!(obb_contained.intersects(&obb));
        assert!(obb.bbox().intersects(&obb_contained.bbox()));
        assert!(obb_contained.bbox().intersects(&obb.bbox()));
    }
}
