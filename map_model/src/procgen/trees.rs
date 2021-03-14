use crate::procgen::heightmap::tree_density;
use flat_spatial::SparseGrid;
use geom::{vec2, Vec2, AABB};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const CELL_SIZE: i32 = 100;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub size: f32,
    pub col: f32,
    pub dir: Vec2,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trees {
    pub grid: SparseGrid<Tree>,
    pub generated: HashSet<(i32, i32)>,
    pub dirt_id: u32,
}

impl Default for Trees {
    fn default() -> Self {
        Self {
            grid: SparseGrid::new(CELL_SIZE),
            generated: Default::default(),
            dirt_id: 1,
        }
    }
}

impl Trees {
    pub fn remove_near_filter(&mut self, bbox: AABB, f: impl Fn(Vec2) -> bool) {
        self.gather_non_generated_chunks(bbox);

        let to_remove: Vec<_> = self
            .grid
            .query_aabb(bbox.ll, bbox.ur)
            .filter(|x| f(x.1))
            .map(|x| x.0)
            .collect();

        self.dirt_id += !to_remove.is_empty() as u32;
        for h in to_remove {
            self.grid.remove(h);
        }

        self.grid.maintain();
    }

    fn cell(p: Vec2) -> (i32, i32) {
        (p.x as i32 / CELL_SIZE, p.y as i32 / CELL_SIZE)
    }
    pub fn gather_non_generated_chunks(&self, aabb: AABB) -> Vec<(i32, i32)> {
        if aabb.h().min(aabb.w()) > 4000.0 {
            return vec![];
        }
        let ll = Self::cell(aabb.ll);
        let ur = Self::cell(aabb.ur);
        let mut not_generated = vec![];
        for y in ll.1..=ur.1 {
            for x in ll.0..=ur.0 {
                let cell = (x, y);
                if !self.generated.contains(&cell) {
                    not_generated.push(cell);
                }
            }
        }
        not_generated
    }

    pub fn add_forest(&mut self, (x, y): (i32, i32)) {
        if !self.generated.insert((x, y)) {
            return;
        }

        let startx = common::rand::rand3(x as f32, y as f32, 0.0);
        let starty = common::rand::rand3(x as f32, y as f32, 1.0);

        let forest_pos = vec2(
            (x as f32 + startx) * CELL_SIZE as f32,
            (y as f32 + starty) * CELL_SIZE as f32,
        );
        let mut active = vec![forest_pos];

        for j in 0..50 {
            if active.is_empty() {
                break;
            }
            let r4 = common::rand::rand3(startx, j as f32, 3.0);
            let idx = (r4 * active.len() as f32) as usize;
            let sample = active[idx];

            let r3 = common::rand::rand3(sample.x, sample.y, j as f32);

            let tdens = tree_density(sample);

            if r3 > tdens * 2.0 {
                active.swap_remove(idx);
                continue;
            }

            for k in 0..5 {
                if k == 9 {
                    active.swap_remove(idx);
                    break;
                }
                let theta = std::f32::consts::TAU * common::rand::rand3(startx, j as f32, k as f32);
                let dist_coeff = 3.0 * common::rand::rand3(startx, j as f32, k as f32 + 10.0);

                let srand = common::rand::rand3(sample.x as f32, sample.y, k as f32);
                let scale = 10.0 + 6.0 * srand;

                let pos = sample + Vec2::from_angle(theta) * (scale * (0.75 + dist_coeff));

                if self.grid.query_around(pos, 0.75 * scale).next().is_some() {
                    continue;
                }

                let crand = common::rand::rand3(pos.x as f32, pos.y, 1.0);

                let colscale = 0.7 - 0.2 * crand;
                let angle = 2.0
                    * std::f32::consts::PI
                    * common::rand::rand3(pos.x as f32, pos.y as f32, 2.0);

                self.grid.insert(
                    pos,
                    Tree {
                        size: scale,
                        col: colscale,
                        dir: Vec2::from_angle(angle),
                    },
                );
                self.dirt_id += 1;

                active.push(pos);
                break;
            }
        }
    }

    pub fn trees(&self) -> impl Iterator<Item = (Vec2, Tree)> + '_ {
        self.grid.handles().map(move |h| {
            let v = self.grid.get(h).unwrap();
            (v.0, *v.1)
        })
    }
}
