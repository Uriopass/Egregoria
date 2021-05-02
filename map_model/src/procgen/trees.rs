use crate::procgen::heightmap::tree_density;
use common::FastSet;
use flat_spatial::storage::Storage;
use flat_spatial::SparseGrid;
use geom::{vec2, Vec2, AABB};
use serde::{Deserialize, Serialize, Serializer};
use std::num::Wrapping;

const CELL_SIZE: i32 = 300;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub size: f32,
    pub col: f32,
    pub dir: Vec2,
}

#[derive(Deserialize, Clone)]
#[serde(from = "SerializedTrees")]
pub struct Trees {
    pub grid: SparseGrid<Tree>,
    pub generated: FastSet<(i32, i32)>,
    pub dirt_id: Wrapping<u32>,
}

impl Default for Trees {
    fn default() -> Self {
        Self {
            grid: SparseGrid::new(CELL_SIZE),
            generated: Default::default(),
            dirt_id: Wrapping(1),
        }
    }
}

impl Trees {
    pub fn remove_near_filter(&mut self, bbox: AABB, f: impl Fn(Vec2) -> bool) {
        self.generate_chunks(bbox);

        let to_remove: Vec<_> = self
            .grid
            .query_aabb(bbox.ll, bbox.ur)
            .filter(|x| f(x.1))
            .map(|x| x.0)
            .collect();

        self.dirt_id += Wrapping(!to_remove.is_empty() as u32);
        for h in to_remove {
            self.grid.remove(h);
        }

        self.grid.maintain();
    }

    fn cell(p: Vec2) -> (i32, i32) {
        (p.x as i32 / CELL_SIZE, p.y as i32 / CELL_SIZE)
    }

    fn chunks_iter(&self, aabb: AABB) -> impl Iterator<Item = (i32, i32)> + '_ {
        let ll = Self::cell(aabb.ll);
        let ur = Self::cell(aabb.ur);
        (ll.1..=ur.1).flat_map(move |y| {
            (ll.0..=ur.0).flat_map(move |x| {
                let cell = (x, y);
                if !self.generated.contains(&cell) {
                    Some(cell)
                } else {
                    None
                }
            })
        })
    }

    pub fn check_non_generated_chunks(&self, aabb: AABB) -> bool {
        self.chunks_iter(aabb).next().is_some()
    }

    pub fn generate_chunks(&mut self, aabb: AABB) {
        log::info!("generating chunks for {:?}", aabb);

        let cells = self.chunks_iter(aabb).collect::<Vec<_>>();
        for cell in cells {
            self.add_forest(cell)
        }
    }

    fn add_forest(&mut self, (x, y): (i32, i32)) {
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

        let cluster_prox = common::rand::rand3(startx, forest_pos.x, forest_pos.y);

        for j in 0..100 {
            if active.is_empty() {
                break;
            }
            let r4 = common::rand::rand3(startx, j as f32, 3.0);
            let idx = (r4 * active.len() as f32) as usize;
            let sample = *unwrap_or!(active.get(idx), break);

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
                let dist_coeff = common::rand::rand3(startx, j as f32, k as f32 + 10.0);

                let pos = sample
                    + Vec2::from_angle(theta)
                        * (4.5 + 45.0 * dist_coeff * dist_coeff * cluster_prox);

                if self.grid.query_around(pos, 10.0).next().is_some() {
                    continue;
                }

                self.grid.insert(pos, Tree::new(pos));
                self.dirt_id += Wrapping(1);

                active.push(pos);
                break;
            }
        }
    }

    pub fn trees(&self) -> impl Iterator<Item = (Vec2, Tree)> + '_ {
        self.grid.objects().map(move |v| (v.0, *v.1))
    }
}

impl Tree {
    pub fn new(pos: Vec2) -> Self {
        let crand = common::rand::rand3(pos.x as f32, pos.y, 1.0);

        let colscale = 0.7 - 0.2 * crand;
        let angle =
            2.0 * std::f32::consts::PI * common::rand::rand3(pos.x as f32, pos.y as f32, 2.0);

        let srand = common::rand::rand3(pos.x as f32, pos.y, 3.0);
        let scale = 7.0 + 2.0 * srand;

        Tree {
            size: scale,
            col: colscale,
            dir: Vec2::from_angle(angle),
        }
    }
}

type SmolTree = u16;

pub fn new_smoltree(pos: Vec2, cell: (i32, i32)) -> SmolTree {
    let diffx = pos.x - (cell.0 * CELL_SIZE) as f32;
    let diffy = pos.y - (cell.1 * CELL_SIZE) as f32;

    ((((diffx / CELL_SIZE as f32) * 256.0) as u8 as u16) << 8)
        + ((diffy / CELL_SIZE as f32) * 256.0) as u8 as u16
}

pub fn to_pos(encoded: SmolTree, cell: (i32, i32)) -> Vec2 {
    let diffx = (encoded >> 8) as u8;
    let diffy = (encoded & 0xFF) as u8;
    Vec2 {
        x: CELL_SIZE as f32 * (cell.0 as f32 + diffx as f32 / 256.0),
        y: CELL_SIZE as f32 * (cell.1 as f32 + diffy as f32 / 256.0),
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedTrees {
    v: Vec<((i32, i32), Vec<SmolTree>)>,
    dirt_id: u32,
}

impl From<SerializedTrees> for Trees {
    fn from(ser: SerializedTrees) -> Self {
        let mut t = Trees {
            dirt_id: Wrapping(ser.dirt_id),
            generated: ser.v.iter().map(|x| x.0).collect(),
            ..Self::default()
        };

        for (cell, v) in ser.v {
            for tree in v {
                let pos = to_pos(tree, cell);
                t.grid.insert(pos, Tree::new(pos));
            }
        }
        t
    }
}

impl Serialize for Trees {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut t = SerializedTrees {
            v: vec![],
            dirt_id: self.dirt_id.0,
        };

        for &cell in &self.generated {
            let gcell = self.grid.storage().cell(cell);
            if let Some(x) = gcell {
                t.v.push((
                    cell,
                    x.objs
                        .iter()
                        .map(move |(_, pos)| new_smoltree(*pos, cell))
                        .collect(),
                ))
            } else {
                t.v.push((cell, vec![]))
            }
        }

        t.serialize(serializer)
    }
}
