use std::ops::Mul;

use flat_spatial::storage::CellIdx;
use flat_spatial::Grid;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use common::FastSet;
use egui_inspect::egui::ahash::HashSetExt;
use geom::{lerp, pack_height, vec2, Intersect, Radians, Ray3, Vec2, Vec3, AABB};
use prototypes::{Tick, DELTA};

use crate::map::procgen::heightmap;
use crate::map::procgen::heightmap::tree_density;

pub type TerrainChunkID = common::ChunkID_512;

pub const TERRAIN_CHUNK_RESOLUTION: usize = 32;

pub(super) const CELL_SIZE: f32 = TerrainChunkID::SIZE_F32 / TERRAIN_CHUNK_RESOLUTION as f32; // 512 / 32 = 16

const TREE_GRID_SIZE: usize = 256;

pub type Chunk = geom::HeightmapChunk<TERRAIN_CHUNK_RESOLUTION, { TerrainChunkID::SIZE }>;
pub type Heightmap = geom::Heightmap<TERRAIN_CHUNK_RESOLUTION, { TerrainChunkID::SIZE }>;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub pos: Vec2,
    pub size: f32,
    pub col: f32,
    pub dir: Vec2,
}

#[derive(Clone)]
pub struct Environment {
    heightmap: Heightmap,
    pub trees: Grid<Tree, Vec2>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerraformKind {
    Elevation,
    Smooth,
    Level,
    Slope,
    Erode,
}

defer_serialize!(Environment, SerializedEnvironment);

impl Default for Environment {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Environment {
    pub fn new(w: u16, h: u16) -> Self {
        let mut me = Self {
            heightmap: Heightmap::new(w, h),
            trees: Grid::new(TREE_GRID_SIZE as i32),
        };
        for y in 0..h {
            let chunks: Vec<_> = (0..w)
                .into_par_iter()
                .map(|x| me.generate_chunk((x, y)))
                .collect();
            for (x, chunk) in (0..w).zip(chunks) {
                if let Some((v, trees)) = chunk {
                    me.heightmap.set_chunk((x, y), v);
                    for tree in trees {
                        me.trees.insert(tree.pos, tree);
                    }
                }
            }
        }
        me
    }

    /// Returns the height of the terrain at the given position in meters, capped at 0
    pub fn height(&self, pos: Vec2) -> Option<f32> {
        self.heightmap.height(pos).map(|x| x.max(0.0))
    }

    /// Returns the height of the terrain at the given position in meters, not capped at 0 (can be negative in water)
    pub fn true_height(&self, pos: Vec2) -> Option<f32> {
        self.heightmap.height(pos)
    }

    pub fn remove_trees_near(
        &mut self,
        obj: impl Intersect<Vec2>,
        mut f: impl FnMut(TerrainChunkID),
    ) {
        let mut to_remove = vec![];

        let bbox = obj.bbox();
        self.trees.query_aabb_visitor(bbox.ll, bbox.ur, |(h, pos)| {
            if obj.intersects(&pos) {
                to_remove.push(h);
            }
        });

        let mut seen = FastSet::new();
        for h in to_remove {
            let Some(tree) = self.trees.remove_maintain(h) else {
                continue;
            };
            let id = TerrainChunkID::new(tree.pos);
            if seen.insert(id) {
                f(id);
            }
        }
    }

    pub fn get_chunk(&self, id: TerrainChunkID) -> Option<&Chunk> {
        self.heightmap.get_chunk((id.0 as u16, id.1 as u16))
    }

    pub fn bounds(&self) -> AABB {
        self.heightmap.bounds()
    }

    /// Returns the size of the map in chunks
    pub fn size(&self) -> (u16, u16) {
        (self.heightmap.w, self.heightmap.h)
    }

    pub fn chunks(&self) -> impl Iterator<Item = (TerrainChunkID, &Chunk)> + '_ {
        self.heightmap
            .chunks()
            .map(|((x, y), c)| (TerrainChunkID::new_i16(x as i16, y as i16), c))
    }

    pub fn covered_chunks(&self, bounds: AABB) -> impl Iterator<Item = TerrainChunkID> {
        self.heightmap
            .covered_chunks(bounds)
            .map(|(x, y)| TerrainChunkID::new_i16(x as i16, y as i16))
    }

    pub fn raycast(&self, ray: Ray3) -> Option<(Vec3, Vec3)> {
        self.heightmap.raycast(ray)
    }

    pub fn set_overrides(
        &mut self,
        chunk: TerrainChunkID,
        overrides: [[u16; TERRAIN_CHUNK_RESOLUTION]; TERRAIN_CHUNK_RESOLUTION],
    ) {
        self.heightmap
            .set_override((chunk.0 as u16, chunk.1 as u16), overrides);
    }

    /// Applies a function to the heightmap
    /// Returns the chunks that were modified
    pub fn terrain_apply(
        &mut self,
        bounds: AABB,
        f: impl FnMut(Vec3) -> f32,
    ) -> Vec<TerrainChunkID> {
        self.heightmap
            .apply(bounds, f)
            .into_iter()
            .map(|(x, y)| TerrainChunkID::new_i16(x as i16, y as i16))
            .collect()
    }

    pub fn terraform(
        &mut self,
        tick: Tick,
        kind: TerraformKind,
        center: Vec2,
        radius: f32,
        amount: f32,
        level: f32,
        slope: Option<(Vec3, Vec3)>,
    ) -> Vec<TerrainChunkID> {
        let bbox = AABB::centered(center, Vec2::splat(radius * 2.0));
        match kind {
            TerraformKind::Elevation => self.terrain_apply(bbox, |pos| {
                let dist = pos.xy().distance(center) / radius;
                if dist >= 1.0 {
                    return pos.z;
                }
                let phi = (-1.0 / (1.0 - dist * dist)).exp();
                pos.z + (amount * DELTA) * phi
            }),
            TerraformKind::Smooth => self
                .heightmap
                .apply_convolution(bbox, |pos, vals| {
                    let dist = pos.distance(center) / radius;
                    if dist >= 1.0 {
                        return vals[4];
                    }
                    let phi = (-1.0 / (1.0 - dist * dist)).exp();

                    const GAUSSIAN_KERNEL: &[f32; 9] = &[
                        0.07511361, 0.1238414, 0.07511361, 0.1238414, 0.20417996, 0.1238414,
                        0.07511361, 0.1238414, 0.07511361,
                    ];
                    let mut sum = 0.0;
                    for (a, b) in vals.iter().zip(GAUSSIAN_KERNEL.iter()) {
                        sum += a * b;
                    }
                    vals[4] + phi * (sum - vals[4])
                })
                .into_iter()
                .map(|(x, y)| TerrainChunkID::new_i16(x as i16, y as i16))
                .collect(),
            TerraformKind::Level => self.terrain_apply(bbox, |pos| {
                let dist = pos.xy().distance(center) / radius;
                if dist >= 1.0 {
                    return pos.z;
                }
                let phi = (-1.0 / (1.0 - dist * dist)).exp();
                pos.z
                    + (amount * DELTA)
                        * phi
                        * (level - pos.z).signum()
                        * (level - pos.z).abs().mul(0.1).clamp(0.0, 1.0)
            }),
            TerraformKind::Slope => self.terrain_apply(bbox, |pos| {
                let dist = pos.xy().distance(center) / radius;
                if dist >= 1.0 {
                    return pos.z;
                }
                let phi = (-1.0 / (1.0 - dist * dist)).exp();
                let mut z = pos.z;
                if let Some((p1, p2)) = slope {
                    let d = p2.xy() - p1.xy();
                    let coeff_along_d = (pos.xy() - p1.xy()).dot(d) / d.mag2();
                    let desired_height = lerp(p1.z, p2.z, coeff_along_d.clamp(0.0, 1.0));

                    z += (amount * DELTA)
                        * phi
                        * (desired_height - pos.z).signum()
                        * (desired_height - pos.z).abs().mul(0.1).clamp(0.0, 1.0);
                }
                z
            }),
            TerraformKind::Erode => {
                let mut rng = common::rand::gen(tick.0);
                let n_particles_continuous = amount * DELTA * radius * radius * 0.00002;
                let mut n_particles = n_particles_continuous as usize;
                if n_particles_continuous.fract() > rng.next_f32() {
                    n_particles += 1;
                }

                self.heightmap
                    .erode(bbox, n_particles, || rng.next_f32())
                    .into_iter()
                    .map(|(x, y)| TerrainChunkID::new_i16(x as i16, y as i16))
                    .collect()
            }
        }
    }

    fn generate_chunk(&self, (x, y): (u16, u16)) -> Option<(Chunk, Vec<Tree>)> {
        let mut heights = [[0; TERRAIN_CHUNK_RESOLUTION]; TERRAIN_CHUNK_RESOLUTION];

        let offchunk = vec2(x as f32, y as f32) * TerrainChunkID::SIZE_F32;
        for (y, l) in heights.iter_mut().enumerate() {
            for (x, h) in l.iter_mut().enumerate() {
                let offcell = vec2(x as f32, y as f32) * CELL_SIZE;
                let mut rh = heightmap::height(offchunk + offcell).0 - 0.12;

                if rh > 0.0 {
                    rh = 0.0;
                }

                *h = pack_height(1000.0 * rh);
            }
        }

        let chunk = Chunk::new(heights);

        let rchunk = common::rand::rand2(x as f32, y as f32);
        let pchunk = TerrainChunkID::SIZE_F32 * vec2(x as f32, y as f32);

        const RES_TREES: usize = 64;
        const TCELLW: f32 = TerrainChunkID::SIZE_F32 / RES_TREES as f32;

        let mut trees = Vec::with_capacity(128);

        let tree_storage = self.trees.storage();

        for offx in 0..RES_TREES {
            for offy in 0..RES_TREES {
                let cellpos = vec2(offx as f32, offy as f32) * TCELLW;

                let rcell = common::rand::rand2(cellpos.x, cellpos.y);
                let jitterx = common::rand::rand3(rchunk, rcell, 1.0);
                let jittery = common::rand::rand3(rchunk, rcell, 2.0);
                let dens_test = common::rand::rand3(rchunk, rcell, 3.0);

                let sample = cellpos + vec2(jitterx, jittery) * TCELLW;

                let tdens = tree_density(pchunk + sample);

                if dens_test < tdens && chunk.height_unchecked(sample) >= 0.0 {
                    let pos = pchunk + sample;
                    // normalize pos
                    let cell = tree_storage.cell_id(pos);
                    let pos = decode_pos(encode_pos(pos, cell), cell);
                    trees.push(Tree::new(pos));
                }
            }
        }

        Some((chunk, trees))
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

pub fn encode_pos(pos: Vec2, chunk: CellIdx) -> SmolTree {
    let diffx = pos.x - (chunk.0 * TREE_GRID_SIZE as i32) as f32;
    let diffy = pos.y - (chunk.1 * TREE_GRID_SIZE as i32) as f32;

    ((((diffx / TREE_GRID_SIZE as f32) * 256.0) as u8 as u16) << 8)
        + ((diffy / TREE_GRID_SIZE as f32) * 256.0) as u8 as u16
}

pub fn decode_pos(encoded: SmolTree, chunk: CellIdx) -> Vec2 {
    let diffx = (encoded >> 8) as u8;
    let diffy = (encoded & 0xFF) as u8;
    Vec2 {
        x: TREE_GRID_SIZE as f32 * (chunk.0 as f32 + diffx as f32 / 256.0),
        y: TREE_GRID_SIZE as f32 * (chunk.1 as f32 + diffy as f32 / 256.0),
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedEnvironment {
    h: Heightmap,
    trees: Vec<(CellIdx, Vec<SmolTree>)>,
}

impl From<SerializedEnvironment> for Environment {
    fn from(ser: SerializedEnvironment) -> Self {
        let mut terrain = Environment {
            heightmap: ser.h,
            ..Self::default()
        };

        for (chunk_id, trees) in ser.trees {
            for tree in trees {
                let tree = Tree::new(decode_pos(tree, chunk_id));
                terrain.trees.insert(tree.pos, tree);
            }
        }
        terrain
    }
}

impl From<&Environment> for SerializedEnvironment {
    fn from(ter: &Environment) -> Self {
        let mut t = SerializedEnvironment {
            h: ter.heightmap.clone(),
            trees: Vec::new(),
        };

        let tree_cells = &ter.trees.storage().cells;

        let mut keys = tree_cells.keys().copied().collect::<Vec<_>>();
        keys.sort_unstable();

        for cell_id in keys {
            let chunk = &tree_cells[&cell_id];
            let mut smoltrees = Vec::with_capacity(chunk.objs.len());
            for (_, tree_pos) in chunk.objs.iter() {
                let smol = encode_pos(*tree_pos, cell_id);
                smoltrees.push(smol);
            }
            t.trees.push((cell_id, smoltrees));
        }

        t
    }
}
