use super::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct AABB3 {
    pub ll: Vec3,
    pub ur: Vec3,
}

impl AABB3 {
    #[inline]
    pub const fn new(ll: Vec3, ur: Vec3) -> Self {
        Self { ll, ur }
    }

    #[inline]
    pub fn centered(pos: Vec3, size: Vec3) -> Self {
        Self {
            ll: pos - size * 0.5,
            ur: pos + size * 0.5,
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
}
