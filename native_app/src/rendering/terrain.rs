use crate::uiworld::UiWorld;
use common::{FastMap, Z_TERRAIN};
use geom::{vec2, LinearColor};
use wgpu_engine::pbuffer::PBuffer;
use wgpu_engine::wgpu::BufferUsage;
use wgpu_engine::IndexType;
use wgpu_engine::{ColoredVertex, FrameContext, GfxContext, Mesh};

const CHUNK_SIZE: f32 = 1000.0;
const RESOLUTION: usize = 20;
struct TerrainChunk {
    mesh: Mesh,
}

pub struct TerrainRender {
    chunks: FastMap<(i32, i32), TerrainChunk>,
    indices: PBuffer,
    n_indices: u32,
}

impl TerrainRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        let (indices, n_indices) = Self::generate_indices(gfx);
        let mut me = TerrainRender {
            chunks: Default::default(),
            indices,
            n_indices: n_indices as u32,
        };

        for y in -10..10 {
            for x in -10..10 {
                me.generate(gfx, x, y);
            }
        }
        me
    }

    fn generate(&mut self, gfx: &mut GfxContext, x: i32, y: i32) {
        let mut mesh = Vec::with_capacity((RESOLUTION + 1) * (RESOLUTION + 1));

        let offset = vec2(x as f32, y as f32) * CHUNK_SIZE;

        for y in 0..=RESOLUTION {
            let y = y as f32 / RESOLUTION as f32;
            for x in 0..=RESOLUTION {
                let x = x as f32 / RESOLUTION as f32;
                let pos = vec2(x, y);
                let pos = pos * CHUNK_SIZE + offset;

                let height = map_model::procgen::heightmap::height(pos);

                let col = if height < 0.1 {
                    common::config().sea_col
                } else if height < 0.12 {
                    common::config().sand_col
                } else {
                    common::config().grass_col
                };
                let col = LinearColor::from(col);

                mesh.push(ColoredVertex {
                    position: [pos.x, pos.y, Z_TERRAIN],
                    color: col.into(),
                })
            }
        }

        let mut vbuf = PBuffer::new(BufferUsage::VERTEX);
        vbuf.write(gfx, bytemuck::cast_slice(&mesh));

        let chunk = TerrainChunk {
            mesh: Mesh {
                vertex_buffer: vbuf.inner().expect("vertices were generated"),
                index_buffer: self.indices.inner().expect("indices was generated"),
                n_indices: self.n_indices,
            },
        };

        self.chunks.insert((x, y), chunk);
    }

    fn generate_indices(gfx: &GfxContext) -> (PBuffer, usize) {
        let mut indices: Vec<IndexType> = Vec::with_capacity(6 * RESOLUTION * RESOLUTION);

        let w = (RESOLUTION + 1) as IndexType;
        for y in 0..RESOLUTION as IndexType {
            for x in 0..RESOLUTION as IndexType {
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
        (buf, l)
    }

    pub fn render(&mut self, _uiw: &UiWorld, fctx: &mut FrameContext) {
        for chunk in self.chunks.values() {
            fctx.objs.push(Box::new(chunk.mesh.clone()))
        }
    }
}
