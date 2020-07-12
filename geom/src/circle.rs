use crate::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        (self.center - other.center).magnitude2() < (self.radius + other.radius).powi(2)
    }
}
