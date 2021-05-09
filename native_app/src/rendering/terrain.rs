use crate::uiworld::UiWorld;
use common::FastMap;
use geom::{vec2, vec3, LinearColor};
use map_model::{Map, CELL_SIZE, CHUNK_RESOLUTION, CHUNK_SIZE};
use std::mem::MaybeUninit;
use std::sync::Arc;
use wgpu_engine::pbuffer::PBuffer;
use wgpu_engine::wgpu::BufferUsage;
use wgpu_engine::{FrameContext, GfxContext, Mesh, Texture};
use wgpu_engine::{IndexType, MeshVertex};

const LOD: usize = 1;

struct TerrainChunk {
    lods: [Mesh; LOD],
    dirt_id: u32,
}

pub struct TerrainRender {
    chunks: FastMap<(i32, i32), TerrainChunk>,
    indices: [(PBuffer, usize); LOD],
    albedo: Arc<Texture>,
    bg: Arc<wgpu_engine::wgpu::BindGroup>,
    dirt_id: u32,
}

impl TerrainRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        let indices = Self::generate_indices(gfx);
        let pal = gfx.palette();
        Self {
            chunks: Default::default(),
            indices,
            bg: Arc::new(pal.bindgroup(&gfx.device, &Texture::bindgroup_layout(&gfx.device))),
            albedo: pal,
            dirt_id: 0,
        }
    }

    pub fn update(&mut self, gfx: &mut GfxContext, map: &Map) {
        if map.terrain.dirt_id.0 != self.dirt_id {
            self.dirt_id = map.terrain.dirt_id.0;

            for &cell in map.terrain.chunks.keys() {
                self.update_chunk(gfx, map, cell)
            }
        }
    }

    fn update_chunk(&mut self, gfx: &mut GfxContext, map: &Map, cell: (i32, i32)) {
        let chunk = unwrap_retlog!(
            map.terrain.chunks.get(&cell),
            "trying to update nonexistent chunk"
        );

        if self
            .chunks
            .get(&cell)
            .map(|x| x.dirt_id == chunk.dirt_id.0)
            .unwrap_or_default()
        {
            return;
        }

        let mut v = vec![];

        let right_chunk = map.terrain.chunks.get(&(cell.0 + 1, cell.1));
        let up_chunk = map.terrain.chunks.get(&(cell.0, cell.1 + 1));
        let upright_chunk = map.terrain.chunks.get(&(cell.0 + 1, cell.1 + 1));

        for lod in 0..LOD {
            let scale = 1 << lod;
            let resolution = CHUNK_RESOLUTION / (1 << lod);

            let mut mesh = Vec::with_capacity((resolution + 1) * (resolution + 1));

            let chunkoff = vec2(
                (cell.0 * CHUNK_SIZE as i32) as f32,
                (cell.1 * CHUNK_SIZE as i32) as f32,
            );

            for y in 0..=resolution {
                for x in 0..=resolution {
                    let fallback = || {
                        chunk.heights[y * scale - (y == resolution) as usize]
                            [x * scale - (x == resolution) as usize]
                    };

                    let getheight = |x: usize, y: usize| match (x >= resolution, y >= resolution) {
                        (false, false) => chunk.heights[y * scale][x * scale],
                        (true, false) => right_chunk
                            .map(|c| c.heights[y * scale][(x - resolution) * scale])
                            .unwrap_or_else(fallback),
                        (false, true) => up_chunk
                            .map(|c| c.heights[(y - resolution) * scale][x * scale])
                            .unwrap_or_else(fallback),
                        (true, true) => upright_chunk
                            .map(|c| c.heights[(y - resolution) * scale][(x - resolution) * scale])
                            .unwrap_or_else(fallback),
                    };

                    let height = getheight(x, y);
                    let hx = getheight(x + 1, y);
                    let hy = getheight(x, y + 1);

                    let pos = chunkoff + vec2(x as f32, y as f32) * CELL_SIZE;

                    let col: LinearColor = if height < -0.02 {
                        common::config().sea_col.into()
                    } else if height < 0.0 {
                        common::config().sand_col.into()
                    } else {
                        0.37 * LinearColor::from(common::config().grass_col)
                    };

                    mesh.push(MeshVertex {
                        position: [pos.x, pos.y, height],
                        normal: vec3(CELL_SIZE * scale as f32, 0.0, hx - height)
                            .cross(vec3(0.0, CELL_SIZE * scale as f32, hy - height))
                            .normalize()
                            .into(),
                        uv: [0.0; 2],
                        color: col.into(),
                    })
                }
            }

            let (ref indice, n_indices) = self.indices[lod];

            let mut vbuf = PBuffer::new(BufferUsage::VERTEX);
            vbuf.write(gfx, bytemuck::cast_slice(&mesh));
            let m = Mesh {
                vertex_buffer: vbuf.inner().expect("vertices were generated"),
                index_buffer: indice.inner().expect("indices was generated"),
                albedo_bg: self.bg.clone(),
                albedo: self.albedo.clone(),
                n_indices: n_indices as u32,
                translucent: false,
            };
            v.push(m);
        }

        let chunk = TerrainChunk {
            lods: collect_arrlod(v),
            dirt_id: chunk.dirt_id.0,
        };
        self.chunks.insert(cell, chunk);
    }

    fn generate_indices(gfx: &GfxContext) -> [(PBuffer, usize); LOD] {
        let mut v = vec![];
        for lod in 0..LOD {
            let resolution = CHUNK_RESOLUTION / (1 << lod);
            let mut indices: Vec<IndexType> = Vec::with_capacity(6 * resolution * resolution);

            let w = (resolution + 1) as IndexType;
            for y in 0..resolution as IndexType {
                for x in 0..resolution as IndexType {
                    let idx = y * w + x;
                    indices.push(idx);
                    indices.push(idx + 1);
                    indices.push(idx + w + 1);

                    indices.push(idx);
                    indices.push(idx + w + 1);
                    indices.push(idx + w);
                }
            }

            let l = indices.len();

            let mut buf = PBuffer::new(BufferUsage::INDEX);
            buf.write(gfx, bytemuck::cast_slice(&indices));
            v.push((buf, l));
        }
        collect_arrlod(v)
    }

    pub fn render(&mut self, _uiw: &UiWorld, fctx: &mut FrameContext) {
        for chunk in self.chunks.values() {
            fctx.objs.push(Box::new(chunk.lods[0].clone()))
        }
    }
}

fn collect_arrlod<T>(x: impl IntoIterator<Item = T>) -> [T; LOD] {
    let mut arr = MaybeUninit::uninit();

    let mut ptr = arr.as_mut_ptr() as *mut T;
    let mut i = 0;
    for v in x {
        if i == LOD {
            panic!("not 4")
        }
        unsafe {
            ptr.write(v);
            ptr = ptr.add(1);
        }
        i += 1;
    }

    if i < LOD {
        panic!("not 4")
    }

    unsafe { arr.assume_init() }
}
