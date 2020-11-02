use crate::procgen::heightmap::height;
use crate::{Map, RoadID};
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

#[derive(Clone)]
pub struct Trees {
    grid: SparseGrid<Tree>,
    pub counter: usize,
    pub dirty: bool,
}

impl Default for Trees {
    fn default() -> Self {
        Self {
            grid: SparseGrid::new(10),
            counter: 1000,
            dirty: true,
        }
    }
}

impl Trees {
    pub fn remove_nearby_trees(map: &mut Map, id: RoadID) {
        let trees = &mut map.trees;
        let r = &map.roads[id];

        let d = r.width + 50.0;
        let mut bbox = r.bbox();
        bbox.x -= d;
        bbox.y -= d;
        bbox.w += d + d;
        bbox.h += d + d;

        let mut to_remove = vec![];
        for (h, tree) in trees
            .grid
            .query_aabb([bbox.x, bbox.y], [bbox.x + bbox.w, bbox.y + bbox.h])
        {
            let rd = common::rand::rand3(tree.x, tree.y, 391.0) * 20.0;

            if r.generated_points
                .project(tree.into())
                .is_close(tree.into(), d - rd)
            {
                to_remove.push(h);
            }
        }

        for h in to_remove {
            trees.grid.remove(h);
            trees.dirty = true;
        }

        trees.grid.maintain();
    }

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
        self.dirty = true;
        self.counter -= 1;
        let i = self.counter as f32;
        let r1 = common::rand::rand2(i, 0.0);
        let r2 = common::rand::rand2(i, 1.0);
        let r3 = common::rand::rand2(i, 2.0);

        let ll = vec2(-6500.0, -6100.0);
        let ur = vec2(5700.0, 3200.0);
        let forest_pos = ll + vec2((ur.x - ll.x) * r1, (ur.y - ll.y) * r2);
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

            let delta_elev = (height(sample) - elev).abs() * 50.0;

            if r3 < sample.distance(forest_pos) / span + delta_elev {
                active.remove(idx);
                continue;
            }

            for k in 0..10 {
                let theta = 2.0 * std::f32::consts::PI * common::rand::rand3(i, j as f32, k as f32);
                let dist = common::rand::rand3(i, j as f32, k as f32 + 10.0);

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
        true
    }

    pub fn from_positions(pos: impl IntoIterator<Item = (Vec2, Tree)>) -> Self {
        let mut grid = SparseGrid::new(20);
        for (v, t) in pos {
            grid.insert(v, t);
        }
        Self {
            grid,
            counter: 0,
            dirty: true,
        }
    }

    pub fn trees(&self) -> impl Iterator<Item = (Vec2, Tree)> + '_ {
        self.grid.handles().map(move |h| {
            let v = self.grid.get(h).unwrap();
            (v.0.into(), *v.1)
        })
    }
}
