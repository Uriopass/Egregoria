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

            let chunk_up = ter.chunks.get(&(cell.0, cell.1 + 1));
            let chunk_down = ter.chunks.get(&(cell.0, cell.1.wrapping_sub(1)));
            let chunk_left = ter.chunks.get(&(cell.0.wrapping_sub(1), cell.1));
            let chunk_right = ter.chunks.get(&(cell.0 + 1, cell.1));

            if self.terrain.update_chunk(
                &mut ctx.gfx,
                cell,
                &chunk.heights,
                |i: usize| {
                    if i >= CRESO {
                        return None;
                    }
                    return Some(chunk_up?.heights[0][i]);
                },
                |i: usize| {
                    if i >= CRESO {
                        return None;
                    }
                    return Some(chunk_down?.heights[CRESO - 1][i]);
                },
                |i: usize| {
                    if i >= CRESO {
                        return None;
                    }
                    return Some(chunk_right?.heights[i][0]);
                },
                |i: usize| {
                    if i >= CRESO {
                        return None;
                    }
                    return Some(chunk_left?.heights[i][CRESO - 1]);
                },
            ) {
                update_count += 1;
                const UPD_PER_FRAME: usize = 20;
                if update_count > UPD_PER_FRAME {
                    break;
                }
            }
        }
    }
}
