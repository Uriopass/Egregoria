use crate::procgen::heightmap::height;
use crate::{Map, RoadID};
use flat_spatial::SparseGrid;
use geom::{vec2, Camera, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const CELL_SIZE: i32 = 1000;

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
    pub dirty: bool,
}

impl Default for Trees {
    fn default() -> Self {
        Self {
            grid: SparseGrid::new(50),
            generated: Default::default(),
            dirty: true,
        }
    }
}

impl Trees {
    pub fn remove_nearby_trees(map: &mut Map, id: RoadID) {
        let trees = &mut map.trees;
        let r = &map.roads[id];

        let d = r.width + 50.0;
        let bbox = r.bbox().expand(d);

        let mut to_remove = vec![];
        for (h, tree) in trees.grid.query_aabb(bbox.ll, bbox.ur) {
            let rd = common::rand::rand3(tree.x, tree.y, 391.0) * 20.0;

            if r.generated_points.project(tree).is_close(tree, d - rd) {
                to_remove.push(h);
            }
        }

        for h in to_remove {
            trees.grid.remove(h);
            trees.dirty = true;
        }

        trees.grid.maintain();
    }

    pub fn update(&mut self, camera: Camera) {
        let aabb = camera.get_screen_box();
        let rpos = aabb.ll + (aabb.ur - aabb.ll) * vec2(rand::random(), rand::random());
        let cell = (rpos.x as i32 / CELL_SIZE, rpos.y as i32 / CELL_SIZE);
        if self.generated.insert(cell) {
            self.add_forest(cell);
        }
    }

    fn add_forest(&mut self, (x, y): (i32, i32)) {
        let startx = common::rand::rand3(x as f32, y as f32, 0.0);
        let starty = common::rand::rand3(x as f32, y as f32, 1.0);
        let r3 = common::rand::rand3(x as f32, y as f32, 4.0);

        let forest_pos = vec2(
            (x as f32 + startx) * CELL_SIZE as f32,
            (y as f32 + starty) * CELL_SIZE as f32,
        );
        let elev = height(forest_pos);

        if elev - 0.15 < r3 * r3 {
            return;
        }
        self.dirty = true;

        let mut active = vec![forest_pos];

        for j in 0..200 {
            if active.is_empty() {
                break;
            }
            let r4 = common::rand::rand3(startx, j as f32, 3.0);
            let idx = (r4 * active.len() as f32) as usize;
            let sample = active[idx];

            let r3 = common::rand::rand3(sample.x, sample.y, 2.0);

            let delta_elev = (height(sample) - elev).abs() * 200.0;

            if r3 < delta_elev {
                active.remove(idx);
                continue;
            }

            for k in 0..10 {
                let theta =
                    2.0 * std::f32::consts::PI * common::rand::rand3(startx, j as f32, k as f32);
                let dist = 50.0 * common::rand::rand3(startx, j as f32, k as f32 + 10.0);

                let srand = common::rand::rand3(sample.x as f32, sample.y, k as f32);
                let scale = 10.0 + 6.0 * srand;

                let pos = sample + Vec2::from_angle(theta) * (scale * 0.5 * (1.0 + dist));

                if self.grid.query_around(pos, scale * 0.5).next().is_some() {
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

                active.push(pos);
            }
            active.remove(idx);
        }
    }

    pub fn trees(&self) -> impl Iterator<Item = (Vec2, Tree)> + '_ {
        self.grid.handles().map(move |h| {
            let v = self.grid.get(h).unwrap();
            (v.0, *v.1)
        })
    }
}
