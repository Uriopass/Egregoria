use crate::aabb::AABB;
use crate::{vec2, Intersect, Segment, Shape, Vec2};
use serde::{Deserialize, Serialize};
use std::hint::unreachable_unchecked;

/// Oriented bounding box
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OBB {
    pub corners: [Vec2; 4],
}

impl OBB {
    /// cossin of UNIT_X makes this an AABB
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

    pub fn axis(&self) -> [Vec2; 2] {
        [
            self.corners[1] - self.corners[0],
            self.corners[3] - self.corners[0],
        ]
    }

    pub fn center(&self) -> Vec2 {
        (self.corners[2] + self.corners[0]) * 0.5
    }

    /// Returns true if other overlaps one dimension of this.
    /// Taken from https://www.flipcode.com/archives/2D_OBB_Intersection.shtml
    fn intersects1way(&self, other: &OBB) -> bool {
        let mut axis = self.axis();

        // Make the length of each axis 1/edge length so we know any
        // dot product must be less than 1 to fall within the edge.
        axis[0] /= axis[0].magnitude2();
        axis[1] /= axis[1].magnitude2();

        let origin = [self.corners[0].dot(axis[0]), self.corners[1].dot(axis[1])];

        for (&axis, &origin) in axis.iter().zip(origin.iter()) {
            let ts = [
                other.corners[0].dot(axis),
                other.corners[1].dot(axis),
                other.corners[2].dot(axis),
                other.corners[3].dot(axis),
            ];

            // Find the extent of box 2 on axis a
            let mut t_min = ts[0];
            let mut t_max = ts[0];

            for &t in &ts[1..4] {
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

    pub fn contains(&self, p: Vec2) -> bool {
        let ok0 = (self.corners[1] - self.corners[0]).dot(p - self.corners[0]) > 0.0;
        let ok1 = (self.corners[2] - self.corners[1]).dot(p - self.corners[1]) > 0.0;
        let ok2 = (self.corners[3] - self.corners[2]).dot(p - self.corners[2]) > 0.0;
        let ok3 = (self.corners[0] - self.corners[3]).dot(p - self.corners[3]) > 0.0;
        ok0 & ok1 & ok2 & ok3
    }

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
}

impl Shape for OBB {
    fn bbox(&self) -> AABB {
        let (min, max) = match super::minmax(&self.corners) {
            Some(x) => x,
            None => unsafe { unreachable_unchecked() },
        };

        AABB::new(min, max)
    }
}

impl Intersect<Vec2> for OBB {
    fn intersects(&self, p: Vec2) -> bool {
        self.contains(p)
    }
}

impl Intersect<OBB> for OBB {
    fn intersects(&self, other: OBB) -> bool {
        self.intersects1way(&other) && other.intersects1way(self)
    }
}

impl Intersect<AABB> for OBB {
    fn intersects(&self, shape: AABB) -> bool {
        OBB {
            corners: [
                shape.ur,
                vec2(shape.ll.x, shape.ur.y),
                shape.ll,
                vec2(shape.ur.x, shape.ll.y),
            ],
        }
        .intersects(*self)
    }
}
