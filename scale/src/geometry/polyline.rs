use super::Vec2;
use crate::geometry::segment::Segment;
use serde::{Deserialize, Serialize};
use std::hint::unreachable_unchecked;
use std::ops::Index;
use std::slice::{Iter, IterMut};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PolyLine(Vec<Vec2>);

impl From<Vec<Vec2>> for PolyLine {
    fn from(x: Vec<Vec2>) -> Self {
        Self(x)
    }
}

impl PolyLine {
    pub fn new(x: Vec<Vec2>) -> Self {
        Self(x)
    }
    pub fn with_capacity(c: usize) -> Self {
        Self(Vec::with_capacity(c))
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

    pub fn last_mut(&mut self) -> Option<&mut Vec2> {
        self.0.last_mut()
    }

    pub fn first_mut(&mut self) -> Option<&mut Vec2> {
        self.0.first_mut()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn reverse(&mut self) {
        self.0.reverse()
    }

    pub fn pop_first(&mut self) -> Option<Vec2> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.remove(0))
        }
    }

    pub fn random_along(&self) -> Option<Vec2> {
        let r: f32 = rand::random();
        match self.n_points() {
            0 => None,
            1 => Some(self[0]),
            2 => Some(self[0] * r + self[1] * (1.0 - r)),
            _ => {
                let l = self.length() * r;
                let mut partial = 0.0;
                for w in self.0.windows(2) {
                    let m = (w[1] - w[0]).magnitude();
                    if partial + m > l {
                        let coeff = (l - partial) / m;
                        return Some(w[0] * coeff + w[1] * (1.0 - coeff));
                    }
                    partial += m;
                }
                None
            }
        }
    }

    pub fn project(&self, p: Vec2) -> Option<Vec2> {
        if self.n_points() <= 1 {
            return self.first();
        }

        if self.n_points() == 2 {
            return Some(
                Segment {
                    a: self.0[0],
                    b: self.0[1],
                }
                .project(p),
            );
        }

        let mut min_proj = vec2!(0.0, 0.0);
        let mut min_dist = std::f32::INFINITY;

        for w in self.0.windows(2) {
            if let [a, b] = w {
                let proj = Segment { a: *a, b: *b }.project(p);
                let d = (p - proj).magnitude2();
                if d <= min_dist {
                    min_dist = d;
                    min_proj = proj;
                }
            } else {
                unsafe { unreachable_unchecked() } // windows(2)
            }
        }
        Some(min_proj)
    }

    pub fn begin_dir(&self) -> Option<Vec2> {
        if self.0.len() >= 2 {
            (self[1] - self[0]).dir_dist().map(|(dir, _)| dir)
        } else {
            None
        }
    }

    pub fn end_dir(&self) -> Option<Vec2> {
        let l = self.0.len();
        if l >= 2 {
            (self[l - 1] - self[l - 2]).dir_dist().map(|(dir, _)| dir)
        } else {
            None
        }
    }

    pub fn length(&self) -> f32 {
        self.0.windows(2).map(|x| (x[1] - x[0]).magnitude()).sum()
    }

    pub fn n_points(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, id: usize) -> Option<&Vec2> {
        self.0.get(id)
    }

    pub fn first(&self) -> Option<Vec2> {
        self.0.first().copied()
    }

    pub fn last(&self) -> Option<Vec2> {
        self.0.last().copied()
    }

    pub fn as_slice(&self) -> &[Vec2] {
        self.0.as_slice()
    }

    pub fn iter(&self) -> Iter<Vec2> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Vec2> {
        self.0.iter_mut()
    }
}

impl Index<usize> for PolyLine {
    type Output = Vec2;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
