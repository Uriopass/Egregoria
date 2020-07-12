use crate::circle::Circle;
use crate::segment::Segment;
use crate::Vec2;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hint::unreachable_unchecked;

#[derive(Clone, Serialize, Deserialize)]
pub struct Polygon(pub Vec<Vec2>);

impl Polygon {
    pub fn contains(&self, p: Vec2) -> bool {
        let nvert = self.0.len();

        let mut j = nvert - 1;
        let mut c = false;

        for i in 0..nvert {
            let verti = self.0[i];
            let vertj = self.0[j];
            let off = vertj - verti;

            let vip = p - verti;
            let vjp = p - vertj;

            if ((vip.y < 0.0) != (vjp.y < 0.0))
                && (vip.x * off.y.abs() < off.x * vip.y * off.y.signum())
            {
                c = !c;
            }
            j = i;
        }
        c
    }

    pub fn project(&self, pos: Vec2) -> Vec2 {
        self.project_segment(pos).0
    }

    pub fn first(&self) -> Vec2 {
        *self.0.first().unwrap()
    }

    pub fn last(&self) -> Vec2 {
        *self.0.last().unwrap()
    }

    pub fn project_segment(&self, p: Vec2) -> (Vec2, usize) {
        match self.0.len() {
            0 => unreachable!(),
            1 => (self.first(), 0),
            2 => (
                Segment {
                    src: self.0[0],
                    dst: self.0[1],
                }
                .project(p),
                1,
            ),
            _ => {
                let l = [self.last(), self.first()];
                self.0
                    .windows(2)
                    .chain(std::iter::once(l.as_ref()))
                    .enumerate()
                    .map(|(i, w)| {
                        if let [a, b] = *w {
                            (Segment { src: a, dst: b }.project(p), i + 1)
                        } else {
                            unsafe { unreachable_unchecked() } // windows(2)
                        }
                    })
                    .min_by_key(|&(proj, _)| OrderedFloat((p - proj).magnitude2()))
                    .unwrap()
            } // Unwrap ok: n_points > 2
        }
    }

    pub fn barycenter(&self) -> Vec2 {
        self.0.iter().sum::<Vec2>() / (self.0.len() as f32)
    }

    pub fn bcircle(&self) -> Circle {
        let center = self.barycenter();
        let radius = self
            .0
            .iter()
            .map(move |x| OrderedFloat(x.distance2(center)))
            .max()
            .unwrap()
            .0
            .sqrt();
        Circle { center, radius }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Vec2> {
        self.0.iter()
    }

    pub fn as_slice(&self) -> &[Vec2] {
        self.0.as_slice()
    }
}
