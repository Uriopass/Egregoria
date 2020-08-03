use crate::rect::Rect;
use crate::Vec2;
use std::hint::unreachable_unchecked;

/// Oriented bounding box
#[derive(Copy, Clone)]
pub struct OBB {
    pub corners: [Vec2; 4],
}

impl OBB {
    /// cossin of UNIT_X makes this an AABB
    pub fn new(center: Vec2, cossin: Vec2, w: f32, h: f32) -> Self {
        let up = cossin * w;
        let right = cossin.perpendicular() * h;
        Self {
            corners: [
                center - up - right,
                center - up + right,
                center + up + right,
                center + up - right,
            ],
        }
    }

    /// Returns true if other overlaps one dimension of this.
    /// Taken from https://www.flipcode.com/archives/2D_OBB_Intersection.shtml
    fn intersects1way(&self, other: &OBB) -> bool {
        let mut axis = [
            self.corners[1] - self.corners[0],
            self.corners[3] - self.corners[0],
        ];

        // Make the length of each axis 1/edge length so we know any
        // dot product must be less than 1 to fall within the edge.
        for x in &mut axis {
            *x /= x.magnitude2();
        }

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

    pub fn bbox(&self) -> Rect {
        let (min, max) = match super::minmax(&self.corners) {
            Some(x) => x,
            None => unsafe { unreachable_unchecked() },
        };

        let diff = max - min;
        Rect::new(min.x, min.y, diff.x, diff.y)
    }

    pub fn intersects(&self, other: OBB) -> bool {
        self.intersects1way(&other) && other.intersects1way(self)
    }
}
