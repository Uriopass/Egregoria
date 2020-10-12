use crate::procgen::heightmap::height;
use flat_spatial::SparseGrid;
use geom::{vec2, Vec2};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Tree {
    pub size: f32,
    pub col: f32,
    pub dir: Vec2,
}

pub struct Trees {
    grid: SparseGrid<Tree>,
    pub counter: usize,
}

impl Default for Trees {
    fn default() -> Self {
        Self {
            grid: SparseGrid::new(10),
            counter: 1000,
        }
    }
}

impl Trees {
    pub fn add_forest(&mut self) -> bool {
        if self.counter == 1 {
            let mut trees = self.trees().collect::<Vec<_>>();

            trees.sort_by_key(|(_, t)| OrderedFloat(t.size));

            *self = Self::from_positions(trees);
            self.counter = 0;
            return true;
        }
        if self.counter <= 1 {
            return false;
        }
        self.counter -= 1;
        let i = self.counter as f32;
        let r1 = common::rand::rand2(i, 0.0);
        let r2 = common::rand::rand2(i, 1.0);
        let r3 = common::rand::rand2(i, 2.0);

        let forest_pos = vec2(-600.0, -1500.0) + vec2(10000.0 * r1, 9000.0 * r2);
        let elev = height(forest_pos);
        if elev - 0.15 < r3 * r3 {
            return false;
        }

        let mut active = vec![forest_pos];

        let span = 1000.0;

        for j in 0..2000 {
            if active.is_empty() {
                break;
            }
            let r4 = common::rand::rand3(i, j as f32, 3.0);
            let idx = (r4 * active.len() as f32) as usize;
            let sample = active[idx];

            let r3 = common::rand::rand3(sample.x, sample.y, 2.0);

            let delta_elev = (height(sample) - elev).abs() * 100.0;

            if r3 < sample.distance(forest_pos) / span + delta_elev {
                active.remove(idx);
                continue;
            }

            for k in 0..10 {
                let theta = 2.0 * std::f32::consts::PI * common::rand::rand3(i, j as f32, k as f32);
                let dist = common::rand::rand3(i, j as f32, k as f32 + 10.0);

                let pos = sample + Vec2::from_angle(theta) * (6.0 + dist * 30.0);

                if self.grid.query_around(pos, 6.0).next().is_some() {
                    continue;
                }

                let srand = common::rand::rand2(pos.x as f32 + 0.391, pos.y as f32 + 0.9381);

                let colscale = 0.7 - 0.2 * srand;
                let scale = 10.0 + 6.0 * srand;
                let angle = 2.0
                    * std::f32::consts::PI
                    * common::rand::rand2(pos.x as f32 + 0.31, pos.y as f32 + 31.9381);

                self.grid.insert(
                    pos,
                    Tree {
                        size: scale,
                        col: colscale,
                        dir: Vec2::from_angle(angle),
                    },
                );

                active.push(pos);
            }
            active.remove(idx);
        }
        true
    }

    pub fn from_positions(pos: impl IntoIterator<Item = (Vec2, Tree)>) -> Self {
        let mut grid = SparseGrid::new(20);
        for (v, t) in pos {
            grid.insert(v, t);
        }
        Self { grid, counter: 0 }
    }

    pub fn trees(&self) -> impl Iterator<Item = (Vec2, Tree)> + '_ {
        self.grid.handles().map(move |h| {
            let v = self.grid.get(h).unwrap();
            (v.0.into(), *v.1)
        })
    }
}
