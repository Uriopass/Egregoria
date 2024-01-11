use super::{Ray3, Sphere, Vec3};
use crate::{Intersect3, Shape3, AABB};
use serde::{Deserialize, Serialize};

#[derive(Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct AABB3 {
    pub ll: Vec3,
    pub ur: Vec3,
}

impl Shape3 for AABB3 {
    fn bbox(&self) -> AABB3 {
        *self
    }
}

impl AABB3 {
    #[inline]
    pub const fn new(ll: Vec3, ur: Vec3) -> Self {
        Self { ll, ur }
    }

    #[inline]
    pub fn new_size(ll: Vec3, size: Vec3) -> Self {
        Self { ll, ur: ll + size }
    }

    #[inline]
    pub fn centered(pos: Vec3, size: Vec3) -> Self {
        Self {
            ll: pos - size * 0.5,
            ur: pos + size * 0.5,
        }
    }

    #[inline]
    pub fn from_aabb(aabb: AABB, height: f32) -> Self {
        Self {
            ll: aabb.ll.z(-height),
            ur: aabb.ur.z(height),
        }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self {
            ll: Vec3::ZERO,
            ur: Vec3::ZERO,
        }
    }

    #[inline]
    pub fn flatten(self) -> AABB {
        AABB {
            ll: self.ll.xy(),
            ur: self.ur.xy(),
        }
    }

    #[inline]
    pub fn w(&self) -> f32 {
        self.ur.x - self.ll.x
    }

    #[inline]
    pub fn h(&self) -> f32 {
        self.ur.y - self.ll.y
    }

    #[inline]
    pub fn union(self, other: Self) -> Self {
        Self {
            ll: self.ll.min(other.ll),
            ur: self.ur.max(other.ur),
        }
    }

    #[inline]
    pub fn union_vec(self, other: Vec3) -> Self {
        Self {
            ll: self.ll.min(other),
            ur: self.ur.max(other),
        }
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        self.ll * 0.5 + self.ur * 0.5
    }

    #[inline]
    pub fn expand(self, w: f32) -> Self {
        Self {
            ll: self.ll - Vec3::splat(w),
            ur: self.ur + Vec3::splat(w),
        }
    }

    #[inline]
    pub fn bounding_sphere(&self) -> Sphere {
        let center = self.center();
        let radius = (self.ur - center).mag();
        Sphere { center, radius }
    }

    #[inline(always)]
    pub fn compute_code(&self, p: Vec3) -> u8 {
        const LEFT: u8 = 1; // 0001
        const RIGHT: u8 = 2; // 0010
        const BOTTOM: u8 = 4; // 0100
        const TOP: u8 = 8; // 1000
        (LEFT * (p.x < self.ll.x) as u8)
            | (RIGHT * (p.x > self.ur.x) as u8)
            | (BOTTOM * (p.y < self.ll.y) as u8)
            | (TOP * (p.y > self.ur.y) as u8)
    }

    #[inline]
    pub fn contains(&self, p: Vec3) -> bool {
        p.x >= self.ll.x
            && p.y >= self.ll.y
            && p.z >= self.ll.z
            && p.x <= self.ur.x
            && p.y <= self.ur.y
            && p.z <= self.ur.z
    }

    #[inline]
    pub fn contains_within(&self, point: Vec3, tolerance: f32) -> bool {
        point.x >= self.ll.x - tolerance
            && point.x <= self.ur.x + tolerance
            && point.y <= self.ur.y + tolerance
            && point.y >= self.ll.y - tolerance
            && point.z <= self.ur.z + tolerance
            && point.z >= self.ll.z - tolerance
    }

    /// as ray is defined by O + tD, return the t values for the entering and exiting intersections
    /// Returns a 2-tuple of (t_near, t_far)
    /// Adapted from <https://gist.github.com/DomNomNom/46bb1ce47f68d255fd5d>
    /// If the ray origin is inside the box, t_near will be zero
    #[inline]
    pub fn raycast(&self, ray: Ray3) -> Option<(f32, f32)> {
        let t_min = (self.ll - ray.from) / ray.dir;
        let t_max = (self.ur - ray.from) / ray.dir;
        let t1 = t_min.min(t_max);
        let t2 = t_min.max(t_max);
        let t_near = f32::max(f32::max(t1.x, t1.y), t1.z);
        let t_far = f32::min(f32::min(t2.x, t2.y), t2.z);
        if t_near >= t_far || t_far < 0.0 {
            return None;
        }
        Some((t_near.max(0.0), t_far))
    }
}

impl Intersect3<AABB3> for AABB3 {
    fn intersects(&self, shape: &AABB3) -> bool {
        self.ll.x <= shape.ur.x
            && self.ur.x >= shape.ll.x
            && self.ll.y <= shape.ur.y
            && self.ur.y >= shape.ll.y
            && self.ll.z <= shape.ur.z
            && self.ur.z >= shape.ll.z
    }
}
