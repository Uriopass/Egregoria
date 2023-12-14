use engine::terrain::TerrainRender as EngineTerrainRender;
use engine::{Context, FrameContext, GfxContext};
use geom::{Camera, InfiniteFrustrum};
use simulation::map::{Map, MapSubscriber, UpdateType};
use simulation::Simulation;

const CSIZE: usize = simulation::map::Heightmap::SIZE as usize;
const CRESO: usize = simulation::map::Heightmap::RESOLUTION;

pub struct TerrainRender {
    terrain: EngineTerrainRender<CSIZE, CRESO>,
    terrain_sub: MapSubscriber,
}

impl TerrainRender {
    pub fn new(gfx: &mut GfxContext, sim: &Simulation) -> Self {
        let (w, h) = sim.map().terrain.size();

        let grass = gfx.texture("assets/sprites/grass.jpg", "grass");

        let terrain = EngineTerrainRender::new(gfx, w as u32, h as u32, grass);

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

        if self.terrain_sub.take_cleared() {
            for (chunk_id, chunk) in ter.chunks() {
                self.terrain.update_chunk(
                    &mut ctx.gfx,
                    (chunk_id.0 as u32, chunk_id.1 as u32),
                    chunk.heights(),
                );
            }

            self.terrain.invalidate_height_normals(&ctx.gfx);
            return;
        }

        let mut update_count = 0;

        while let Some(cell) = self.terrain_sub.take_one_updated_chunk() {
            for chunkid in cell.convert() {
                let chunk =
                    unwrap_retlog!(ter.get_chunk(chunkid), "trying to update nonexistent chunk");

                self.terrain.update_chunk(
                    &mut ctx.gfx,
                    (chunkid.0 as u32, chunkid.1 as u32),
                    chunk.heights(),
                );
            }

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
