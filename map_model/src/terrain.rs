use crate::procgen::heightmap::tree_density;
use geom::{vec2, Vec2, AABB};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::num::Wrapping;

pub const CHUNK_SIZE: u32 = 300;
pub const CHUNK_RESOLUTION: usize = 10;

#[derive(Clone)]
pub struct Chunk {
    pub trees: Vec<Tree>,
    pub heights: [[f32; CHUNK_RESOLUTION]; CHUNK_RESOLUTION],
    pub dirt_id: Wrapping<u32>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            trees: Default::default(),
            heights: Default::default(),
            dirt_id: Wrapping(1),
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub pos: Vec2,
    pub size: f32,
    pub col: f32,
    pub dir: Vec2,
}

#[derive(Deserialize, Clone)]
#[serde(from = "SerializedTerrain")]
pub struct Terrain {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub dirt_id: Wrapping<u32>,
}

impl Default for Terrain {
    fn default() -> Self {
        Self::new()
    }
}

impl Terrain {
    pub fn new() -> Self {
        Self {
            chunks: Default::default(),
            dirt_id: Wrapping(1),
        }
    }

    pub fn remove_near_filter(&mut self, bbox: AABB, f: impl Fn(Vec2) -> bool) {
        let mut v = false;
        for cell in self.chunks_iter(bbox) {
            let chunk = unwrap_cont!(self.chunks.get_mut(&cell));
            let mut vcell = false;
            chunk.trees.retain(|t| {
                let b = !f(t.pos);
                vcell |= !b;
                b
            });
            chunk.dirt_id += Wrapping(vcell as u32);
            v |= vcell;
        }
        self.dirt_id += Wrapping(v as u32)
    }

    pub fn cell(p: Vec2) -> (i32, i32) {
        (
            p.x as i32 / CHUNK_SIZE as i32,
            p.y as i32 / CHUNK_SIZE as i32,
        )
    }

    fn chunks_iter(&self, aabb: AABB) -> impl Iterator<Item = (i32, i32)> {
        let ll = Self::cell(aabb.ll);
        let ur = Self::cell(aabb.ur);
        (ll.1..=ur.1).flat_map(move |y| (ll.0..=ur.0).map(move |x| (x, y)))
    }

    pub fn generate_chunk(&mut self, (x, y): (i32, i32)) {
        if self.chunks.contains_key(&(x, y)) {
            return;
        }

        let chunk = self.chunks.entry((x, y)).or_default();

        let offchunk = vec2(x as f32, y as f32) * CHUNK_SIZE as f32;
        for (y, l) in chunk.heights.iter_mut().enumerate() {
            for (x, h) in l.iter_mut().enumerate() {
                let offcell =
                    vec2(x as f32, y as f32) / CHUNK_RESOLUTION as f32 * CHUNK_SIZE as f32;
                *h = crate::procgen::heightmap::height(offchunk + offcell).0;
            }
        }

        let rchunk = common::rand::rand2(x as f32, y as f32);
        let pchunk = CHUNK_SIZE as f32 * vec2(x as f32, y as f32);

        const RES_TREES: usize = 10;
        const TCELLW: f32 = CHUNK_SIZE as f32 / RES_TREES as f32;

        for offx in 0..RES_TREES {
            for offy in 0..RES_TREES {
                let rcell = common::rand::rand2(offx as f32, offy as f32);

                let jitterx = common::rand::rand3(rchunk, rcell, 1.0);
                let jittery = common::rand::rand3(rchunk, rcell, 2.0);
                let dens_test = common::rand::rand3(rchunk, rcell, 3.0);

                let sample = pchunk
                    + vec2(offx as f32, offy as f32) * TCELLW
                    + vec2(jitterx, jittery) * TCELLW;

                let tdens = tree_density(sample);

                if dens_test > tdens * 2.0 {
                    continue;
                }

                chunk.trees.push(Tree::new(sample));
            }
        }
    }

    pub fn trees(&self) -> impl Iterator<Item = &Tree> + '_ {
        self.chunks.values().flat_map(|x| &x.trees)
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
            pos,
            size: scale,
            col: colscale,
            dir: Vec2::from_angle(angle),
        }
    }
}

type SmolTree = u16;

pub fn new_smoltree(pos: Vec2, cell: (i32, i32)) -> SmolTree {
    let diffx = pos.x - (cell.0 * CHUNK_SIZE as i32) as f32;
    let diffy = pos.y - (cell.1 * CHUNK_SIZE as i32) as f32;

    ((((diffx / CHUNK_SIZE as f32) * 256.0) as u8 as u16) << 8)
        + ((diffy / CHUNK_SIZE as f32) * 256.0) as u8 as u16
}

pub fn to_pos(encoded: SmolTree, cell: (i32, i32)) -> Vec2 {
    let diffx = (encoded >> 8) as u8;
    let diffy = (encoded & 0xFF) as u8;
    Vec2 {
        x: CHUNK_SIZE as f32 * (cell.0 as f32 + diffx as f32 / 256.0),
        y: CHUNK_SIZE as f32 * (cell.1 as f32 + diffy as f32 / 256.0),
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedChunk {
    trees: Vec<SmolTree>,
    heights: [[f32; CHUNK_RESOLUTION]; CHUNK_RESOLUTION],
}

#[derive(Serialize, Deserialize)]
struct SerializedTerrain {
    v: Vec<((i32, i32), SerializedChunk)>,
    dirt_id: u32,
}

impl From<SerializedTerrain> for Terrain {
    fn from(ser: SerializedTerrain) -> Self {
        let mut t = Terrain {
            dirt_id: Wrapping(ser.dirt_id),
            ..Self::default()
        };

        for (cell, v) in ser.v {
            let trees = v
                .trees
                .into_iter()
                .map(|x| Tree::new(to_pos(x, cell)))
                .collect();

            t.chunks.insert(
                cell,
                Chunk {
                    trees,
                    heights: v.heights,
                    dirt_id: Wrapping(1),
                },
            );
        }
        t
    }
}

impl Serialize for Terrain {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut t = SerializedTerrain {
            v: vec![],
            dirt_id: self.dirt_id.0,
        };

        for (&cell, chunk) in &self.chunks {
            t.v.push((
                cell,
                SerializedChunk {
                    trees: chunk
                        .trees
                        .iter()
                        .map(move |tree| new_smoltree(tree.pos, cell))
                        .collect(),
                    heights: chunk.heights,
                },
            ))
        }

        t.serialize(serializer)
    }
}
