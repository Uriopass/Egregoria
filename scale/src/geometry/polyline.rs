use super::Vec2;
use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PolyLine(pub Vec<Vec2>);

impl From<Vec<Vec2>> for PolyLine {
    fn from(x: Vec<Vec2>) -> Self {
        Self(x)
    }
}

impl PolyLine {
    pub fn with_capacity(c: usize) -> Self {
        Self(Vec::with_capacity(c))
    }

    pub fn length(&self) -> f32 {
        self.0.windows(2).map(|x| (x[1] - x[0]).magnitude()).sum()
    }

    pub fn n_points(&self) -> usize {
        self.0.len()
    }

    pub fn extend<'a>(&mut self, s: impl IntoIterator<Item = &'a Vec2>) {
        self.0.extend(s)
    }

    pub fn pop(&mut self) -> Option<Vec2> {
        self.0.pop()
    }

    pub fn push(&mut self, item: Vec2) {
        self.0.push(item)
    }

    pub fn last(&self) -> Option<&Vec2> {
        self.0.last()
    }

    pub fn first(&self) -> Option<&Vec2> {
        self.0.first()
    }

    pub fn as_slice(&self) -> &[Vec2] {
        &self.0
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

impl Index<usize> for PolyLine {
    type Output = Vec2;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
