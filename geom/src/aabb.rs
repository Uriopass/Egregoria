use super::Vec2;
use crate::{Circle, Intersect, Segment, Shape};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct AABB {
    pub ll: Vec2,
    pub ur: Vec2,
}

impl AABB {
    /// Create a new `AABB`.
    pub const fn new(ll: Vec2, ur: Vec2) -> Self {
        AABB { ll, ur }
    }

    /// Create a new `AABB` with all values zero.
    pub const fn zero() -> Self {
        Self {
            ll: Vec2::ZERO,
            ur: Vec2::ZERO,
        }
    }

    pub fn w(&self) -> f32 {
        self.ur.x - self.ll.x
    }

    pub fn h(&self) -> f32 {
        self.ur.y - self.ll.y
    }

    pub fn union(self, other: AABB) -> AABB {
        AABB {
            ll: self.ll.min(other.ll),
            ur: self.ur.max(other.ur),
        }
    }

    pub fn center(&self) -> Vec2 {
        self.ll * 0.5 + self.ur * 0.5
    }

    pub fn expand(self, w: f32) -> Self {
        Self {
            ll: self.ll - Vec2::splat(w),
            ur: self.ur - Vec2::splat(w),
        }
    }

    fn compute_code(&self, p: Vec2) -> u8 {
        const INSIDE: u8 = 0; // 0000
        const LEFT: u8 = 1; // 0001
        const RIGHT: u8 = 2; // 0010
        const BOTTOM: u8 = 4; // 0100
        const TOP: u8 = 8; // 1000
        let mut code = INSIDE; // initialised as being inside of [[clip window]]

        if p.x < self.ll.x {
            // to the left of clip window
            code |= LEFT;
        } else if p.x > self.ur.x {
            // to the right of clip window
            code |= RIGHT;
        }

        if p.y < self.ll.y {
            // below the clip window
            code |= BOTTOM;
        } else if p.y > self.ur.y {
            // above the clip window
            code |= TOP;
        }
        code
    }

    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.ll.x && p.y >= self.ll.y && p.x <= self.ur.x && p.y <= self.ur.y
    }

    /// Checks whether the `AABB` contains a `Point`
    pub fn contains_within(&self, point: Vec2, tolerance: f32) -> bool {
        point.x >= self.ll.x - tolerance
            && point.x <= self.ur.x + tolerance
            && point.y <= self.ur.y + tolerance
            && point.y >= self.ll.y - tolerance
    }

    pub fn segments(&self) -> impl Iterator<Item = Segment> {
        let ul = Vec2 {
            x: self.ll.x,
            y: self.ur.y,
        };
        let lr = Vec2 {
            x: self.ur.x,
            y: self.ll.y,
        };
        let ll = self.ll;
        let ur = self.ur;

        std::iter::once(Segment::new(ll, lr))
            .chain(std::iter::once(Segment::new(lr, ur)))
            .chain(std::iter::once(Segment::new(ur, ul)))
            .chain(std::iter::once(Segment::new(ul, ll)))
    }
}

impl Shape for AABB {
    fn bbox(&self) -> AABB {
        *self
    }
}

impl Intersect<AABB> for AABB {
    fn intersects(&self, b: &AABB) -> bool {
        let a = self;
        let x =
            f32::abs((a.ll.x + a.ur.x) - (b.ll.x + b.ur.x)) <= (a.ur.x - a.ll.x + b.ur.x - b.ll.x);
        let y =
            f32::abs((a.ll.y + a.ur.y) - (b.ll.y + b.ur.y)) <= (a.ur.y - a.ll.y + b.ur.y - b.ll.y);

        x & y
    }
}

impl Intersect<Circle> for AABB {
    fn intersects(&self, c: &Circle) -> bool {
        c.intersects(self)
    }
}

impl Intersect<Segment> for AABB {
    fn intersects(&self, s: &Segment) -> bool {
        let outcode0 = self.compute_code(s.src);
        let outcode1 = self.compute_code(s.dst);
        if outcode0 == 0 || outcode1 == 0 {
            return true;
        }
        if outcode0 & outcode1 != 0 {
            return false;
        }
        self.segments().any(move |seg| seg.intersects(s))
    }
}

impl Intersect<Vec2> for AABB {
    fn intersects(&self, p: &Vec2) -> bool {
        self.contains(*p)
    }
}
