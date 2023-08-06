use crate::map::procgen::heightmap;
use crate::map::procgen::heightmap::tree_density;
use geom::{vec2, Intersect, Radians, Vec2, AABB};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CHUNK_SIZE: u32 = 1024;
pub const CHUNK_RESOLUTION: usize = 32;
pub const CELL_SIZE: f32 = CHUNK_SIZE as f32 / CHUNK_RESOLUTION as f32;

#[derive(Clone)]
pub struct Chunk {
    pub trees: Vec<Tree>,
    pub heights: [[f32; CHUNK_RESOLUTION]; CHUNK_RESOLUTION],
}

impl Chunk {
    pub fn rect(id: ChunkID) -> AABB {
        let ll = vec2(
            id.0 as f32 * CHUNK_SIZE as f32,
            id.1 as f32 * CHUNK_SIZE as f32,
        );
        let ur = ll + vec2(CHUNK_SIZE as f32, CHUNK_SIZE as f32);
        AABB::new(ll, ur)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            trees: Default::default(),
            heights: Default::default(),
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

pub type ChunkID = (u32, u32);

pub fn chunk_id(v: Vec2) -> ChunkID {
    if v.x < 0.0 || v.y < 0.0 {
        return (0, 0);
    }
    (v.x as u32 / CHUNK_SIZE, v.y as u32 / CHUNK_SIZE)
}

#[derive(Clone)]
pub struct Terrain {
    pub chunks: BTreeMap<ChunkID, Chunk>,
    pub width: u32,
    pub height: u32,
}

defer_serialize!(Terrain, SerializedTerrain);

impl Default for Terrain {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Terrain {
    pub fn new(w: u32, h: u32) -> Self {
        let mut me = Self {
            chunks: Default::default(),
            width: w,
            height: h,
        };
        for y in 0..h {
            let chunks: Vec<_> = (0..w)
                .into_par_iter()
                .map(|x| me.generate_chunk((x, y)))
                .collect();
            for (x, chunk) in (0..w).zip(chunks) {
                if let Some(v) = chunk {
                    me.chunks.insert((x, y), v);
                }
            }
        }
        me
    }

    pub fn remove_near(&mut self, obj: impl Intersect<Vec2>, mut f: impl FnMut(ChunkID)) {
        for cell in self.chunks_iter(obj.bbox()) {
            let chunk = unwrap_cont!(self.chunks.get_mut(&cell));
            let mut changed = false;
            chunk.trees.retain(|t| {
                let delete_tree = obj.intersects(&t.pos);
                changed |= delete_tree;
                !delete_tree
            });
            if changed {
                f(cell);
            }
        }
    }

    pub fn cell(p: Vec2) -> (u32, u32) {
        chunk_id(p)
    }

    fn chunks_iter(&self, aabb: AABB) -> impl Iterator<Item = (u32, u32)> {
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

    pub fn generate_chunk(&self, (x, y): (u32, u32)) -> Option<Chunk> {
        if self.chunks.contains_key(&(x, y)) {
            return None;
        }

        let mut chunk = Chunk::default();

        let offchunk = vec2(x as f32, y as f32) * CHUNK_SIZE as f32;
        for (y, l) in chunk.heights.iter_mut().enumerate() {
            for (x, h) in l.iter_mut().enumerate() {
                let offcell = vec2(x as f32, y as f32) * CELL_SIZE;
                let mut rh = heightmap::height(offchunk + offcell).0 - 0.12;

                if rh > 0.0 {
                    rh = 0.0;
                }

                *h = 1000.0 * rh;
            }
        }

        let rchunk = common::rand::rand2(x as f32, y as f32);
        let pchunk = CHUNK_SIZE as f32 * vec2(x as f32, y as f32);

        const RES_TREES: usize = 64;
        const TCELLW: f32 = CHUNK_SIZE as f32 / RES_TREES as f32;

        for offx in 0..RES_TREES {
            for offy in 0..RES_TREES {
                let cellpos = vec2(offx as f32, offy as f32) * TCELLW;

                let rcell = common::rand::rand2(cellpos.x, cellpos.y);
                let jitterx = common::rand::rand3(rchunk, rcell, 1.0);
                let jittery = common::rand::rand3(rchunk, rcell, 2.0);
                let dens_test = common::rand::rand3(rchunk, rcell, 3.0);

                let sample = cellpos + vec2(jitterx, jittery) * TCELLW;

                let tdens = tree_density(pchunk + sample);

                if dens_test < tdens && chunk.height(sample) >= 0.0 {
                    chunk.trees.push(Tree::new(pchunk + sample));
                }
            }
        }

        Some(chunk)
    }
}

impl Chunk {
    pub fn height(&self, p: Vec2) -> f32 {
        let v = p / CHUNK_SIZE as f32;
        let v = v * CHUNK_RESOLUTION as f32;
        self.heights[v.y as usize][v.x as usize]
    }
}

impl Tree {
    pub fn new(pos: Vec2) -> Self {
        let crand = common::rand::rand3(pos.x, pos.y, 1.0);

        let colscale = 0.7 - 0.2 * crand;
        let angle = Radians(2.0 * std::f32::consts::PI * common::rand::rand3(pos.x, pos.y, 2.0));

        let srand = common::rand::rand3(pos.x, pos.y, 3.0);
        let scale = 5.0 + 3.0 * srand;

        Tree {
            pos,
            size: scale,
            col: colscale,
            dir: angle.vec2(),
        }
    }
}

type SmolTree = u16;

pub fn new_smoltree(pos: Vec2, chunk: (u32, u32)) -> SmolTree {
    let diffx = pos.x - (chunk.0 * CHUNK_SIZE) as f32;
    let diffy = pos.y - (chunk.1 * CHUNK_SIZE) as f32;

    ((((diffx / CHUNK_SIZE as f32) * 256.0) as u8 as u16) << 8)
        + ((diffy / CHUNK_SIZE as f32) * 256.0) as u8 as u16
}

pub fn to_pos(encoded: SmolTree, chunk: (u32, u32)) -> Vec2 {
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
    v: Vec<((u32, u32), SerializedChunk)>,
    w: u32,
    h: u32,
}

impl From<SerializedTerrain> for Terrain {
    fn from(ser: SerializedTerrain) -> Self {
        let mut t = Terrain {
            width: ser.w,
            height: ser.h,
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
                },
            );
        }
        t
    }
}

impl From<&Terrain> for SerializedTerrain {
    fn from(ter: &Terrain) -> Self {
        let mut t = SerializedTerrain {
            v: vec![],
            w: ter.width,
            h: ter.height,
        };

        for (&cell, chunk) in &ter.chunks {
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

        t
    }
}
