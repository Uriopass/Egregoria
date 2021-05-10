use crate::procgen::heightmap::tree_density;
use geom::{vec2, Vec2, AABB};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::num::Wrapping;

pub const CHUNK_SIZE: u32 = 1024;
pub const CHUNK_RESOLUTION: usize = 32;
pub const CELL_SIZE: f32 = CHUNK_SIZE as f32 / CHUNK_RESOLUTION as f32;

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

    pub fn remove_near_filter(&mut self, bbox: AABB, should_remove: impl Fn(Vec2) -> bool) {
        let mut v = false;
        for cell in self.chunks_iter(bbox) {
            let chunk = unwrap_cont!(self.chunks.get_mut(&cell));
            let mut vcell = false;
            chunk.trees.retain(|t| {
                let rem = should_remove(t.pos);
                vcell |= rem;
                !rem
            });
            chunk.dirt_id += Wrapping(vcell as u32);
            v |= vcell;
        }
        self.dirt_id += Wrapping(v as u32)
    }

    pub fn cell(p: Vec2) -> (i32, i32) {
        (
            p.x as i32 / CHUNK_SIZE as i32 - if p.x < 0.0 { 1 } else { 0 },
            p.y as i32 / CHUNK_SIZE as i32 - if p.y < 0.0 { 1 } else { 0 },
        )
    }

    fn chunks_iter(&self, aabb: AABB) -> impl Iterator<Item = (i32, i32)> {
        let ll = Self::cell(aabb.ll);
        let ur = Self::cell(aabb.ur);
        (ll.1..=ur.1).flat_map(move |y| (ll.0..=ur.0).map(move |x| (x, y)))
    }

    pub fn height(&self, p: Vec2) -> Option<f32> {
        let exact = self.height_nearest(p);
        if let (Some(a), Some(b), Some(c), Some(d)) = (
            exact,
            self.height_nearest(p + Vec2::x(CELL_SIZE)),
            self.height_nearest(p + Vec2::y(CELL_SIZE)),
            self.height_nearest(p + vec2(CELL_SIZE, CELL_SIZE)),
        ) {
            return Some((a + b + c + d) / 4.0);
        }
        exact
    }

    fn height_nearest(&self, p: Vec2) -> Option<f32> {
        let cell = Self::cell(p);
        self.chunks.get(&cell).and_then(|chunk| {
            let v = p / CHUNK_SIZE as f32 - vec2(cell.0 as f32, cell.1 as f32);
            let v = v * CHUNK_RESOLUTION as f32;
            chunk
                .heights
                .get(v.y as usize)
                .and_then(|x| x.get(v.x as usize))
                .copied()
        })
    }

    pub fn generate_chunk(&mut self, (x, y): (i32, i32)) {
        if self.chunks.contains_key(&(x, y)) {
            return;
        }

        let chunk = self.chunks.entry((x, y)).or_default();

        let offchunk = vec2(x as f32, y as f32) * CHUNK_SIZE as f32;
        for (y, l) in chunk.heights.iter_mut().enumerate() {
            for (x, h) in l.iter_mut().enumerate() {
                let offcell = vec2(x as f32, y as f32) * CELL_SIZE;
                *h = 1000.0
                    * (crate::procgen::heightmap::height(offchunk + offcell).0 - 0.12).min(0.0);
            }
        }

        let rchunk = common::rand::rand2(x as f32, y as f32);
        let pchunk = CHUNK_SIZE as f32 * vec2(x as f32, y as f32);

        const RES_TREES: usize = 32;
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

                if dens_test < tdens * 2.0 - 0.3 {
                    chunk.trees.push(Tree::new(sample));
                }
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

pub fn new_smoltree(pos: Vec2, chunk: (i32, i32)) -> SmolTree {
    let diffx = pos.x - (chunk.0 * CHUNK_SIZE as i32) as f32;
    let diffy = pos.y - (chunk.1 * CHUNK_SIZE as i32) as f32;

    ((((diffx / CHUNK_SIZE as f32) * 256.0) as u8 as u16) << 8)
        + ((diffy / CHUNK_SIZE as f32) * 256.0) as u8 as u16
}

pub fn to_pos(encoded: SmolTree, chunk: (i32, i32)) -> Vec2 {
    let diffx = (encoded >> 8) as u8;
    let diffy = (encoded & 0xFF) as u8;
    Vec2 {
        x: CHUNK_SIZE as f32 * (chunk.0 as f32 + diffx as f32 / 256.0),
        y: CHUNK_SIZE as f32 * (chunk.1 as f32 + diffy as f32 / 256.0),
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

        for (chunk_pos, v) in ser.v {
            let trees = v
                .trees
                .into_iter()
                .map(|x| Tree::new(to_pos(x, chunk_pos)))
                .collect();

            t.chunks.insert(
                chunk_pos,
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
