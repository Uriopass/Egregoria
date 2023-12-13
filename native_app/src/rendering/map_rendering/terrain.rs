use engine::terrain::TerrainRender as EngineTerrainRender;
use engine::{Context, FrameContext, GfxContext};
use geom::{Camera, InfiniteFrustrum};
use simulation::map::{Map, MapSubscriber, UpdateType, CHUNK_RESOLUTION, CHUNK_SIZE};
use simulation::Simulation;

const CSIZE: usize = CHUNK_SIZE as usize;
const CRESO: usize = CHUNK_RESOLUTION;

pub struct TerrainRender {
    terrain: EngineTerrainRender<CSIZE, CRESO>,
    terrain_sub: MapSubscriber,
}

impl TerrainRender {
    pub fn new(gfx: &mut GfxContext, sim: &Simulation) -> Self {
        let w = sim.map().terrain.width;
        let h = sim.map().terrain.height;

        let grass = gfx.texture("assets/sprites/grass.jpg", "grass");

        let terrain = EngineTerrainRender::new(gfx, w, h, grass);

        Self {
            terrain,
            terrain_sub: sim.map().subscribe(UpdateType::Terrain),
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

            self.terrain
                .update_chunk(&mut ctx.gfx, cell, &chunk.heights);

            update_count += 1;
            if update_count > 20 {
                break;
            }
        }

        if update_count > 0 {
            self.terrain.invalidate_height_normals(&ctx.gfx);
        }
    }
}
