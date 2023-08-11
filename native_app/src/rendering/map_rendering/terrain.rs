use crate::context::Context;
use egregoria::map::{Map, MapSubscriber, UpdateType, CHUNK_RESOLUTION, CHUNK_SIZE};
use egregoria::Egregoria;
use geom::{Camera, InfiniteFrustrum};
use wgpu_engine::terrain::TerrainRender as EngineTerrainRender;
use wgpu_engine::{FrameContext, GfxContext};

const CSIZE: usize = CHUNK_SIZE as usize;
const CRESO: usize = CHUNK_RESOLUTION;

pub struct TerrainRender {
    terrain: EngineTerrainRender<CSIZE, CRESO>,
    terrain_sub: MapSubscriber,
}

impl TerrainRender {
    pub fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let w = goria.map().terrain.width;
        let h = goria.map().terrain.height;

        let grass = gfx.texture("assets/sprites/grass.jpg", "grass");

        let terrain =
            EngineTerrainRender::new(gfx, w, h, egregoria::config().border_col.into(), grass);

        /*
        let ter = &goria.map().terrain;
        let minchunk = *ter.chunks.keys().min().unwrap();
        let maxchunk = *ter.chunks.keys().max().unwrap();
        terrain.update_borders(minchunk, maxchunk, gfx, &|p| ter.height(p));
         */

        Self {
            terrain,
            terrain_sub: goria.map().subscribe(UpdateType::Terrain),
        }
    }

    pub fn draw(&mut self, cam: &Camera, frustrum: &InfiniteFrustrum, fctx: &mut FrameContext<'_>) {
        self.terrain.draw_terrain(cam, frustrum, fctx);
    }

    pub fn update(&mut self, ctx: &mut Context, map: &Map) {
        let ter = &map.terrain;

        let mut update_count = 0;
        while let Some(cell) = self.terrain_sub.take_one_updated_chunk() {
            let chunk = unwrap_retlog!(ter.chunks.get(&cell), "trying to update nonexistent chunk");

            if self
                .terrain
                .update_chunk(&mut ctx.gfx, cell, &chunk.heights)
            {
                update_count += 1;
                #[cfg(not(debug_assertions))]
                const UPD_PER_FRAME: usize = 20;

                #[cfg(debug_assertions)]
                const UPD_PER_FRAME: usize = 8;
                if update_count > UPD_PER_FRAME {
                    break;
                }
            }
        }
    }
}
