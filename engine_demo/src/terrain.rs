use crate::DemoElement;
use engine::terrain::TerrainRender as EngineTerrainRender;
use engine::{Context, FrameContext};
use geom::{vec2, Camera, InfiniteFrustrum};

const CSIZE: usize = 512;
const CRESO: usize = 32;
const MAP_SIZE: usize = 25;

pub struct Terrain {
    terrain: EngineTerrainRender<CSIZE, CRESO>,
    heights: Box<[[[[f32; CRESO]; CRESO]; MAP_SIZE]; MAP_SIZE]>,
}

impl DemoElement for Terrain {
    fn name(&self) -> &'static str {
        "Terrain"
    }

    fn init(ctx: &mut Context) -> Self {
        let gfx = &mut ctx.gfx;

        let mut heights: Box<[[[[f32; CRESO]; CRESO]; MAP_SIZE]; MAP_SIZE]> =
            vec![[[[0.0; CRESO]; CRESO]; MAP_SIZE]; MAP_SIZE]
                .into_boxed_slice()
                .try_into()
                .unwrap();

        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                for i in 0..CRESO {
                    for j in 0..CRESO {
                        heights[y][x][i][j] = 600.0
                            * (0.5
                                + geom::fnoise(
                                    0.01 * vec2((x * CRESO + j) as f32, (y * CRESO + i) as f32),
                                )
                                .0
                                .powi(2));
                    }
                }
            }
        }

        let grass = gfx.texture("assets/sprites/grass.jpg", "grass");

        let mut terrain = EngineTerrainRender::new(gfx, MAP_SIZE as u32, MAP_SIZE as u32, grass);

        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                terrain.update_chunk(
                    gfx,
                    (x as u32, y as u32),
                    &heights[y][x],
                    |j: usize| {
                        if y + 1 == MAP_SIZE || j >= CRESO {
                            return None;
                        }
                        Some(heights[y + 1][x][0][j])
                    },
                    |j: usize| {
                        if y == 0 || j >= CRESO {
                            return None;
                        }
                        Some(heights[y - 1][x][CRESO - 1][j])
                    },
                    |i: usize| {
                        if x + 1 == MAP_SIZE || i >= CRESO {
                            return None;
                        }
                        Some(heights[y][x + 1][i][0])
                    },
                    |i: usize| {
                        if x == 0 || i >= CRESO {
                            return None;
                        }
                        Some(heights[y][x - 1][i][CRESO - 1])
                    },
                );
            }
        }

        Self { terrain, heights }
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn render(&mut self, fc: &mut FrameContext, cam: &Camera, frustrum: &InfiniteFrustrum) {
        self.terrain.draw_terrain(cam, frustrum, fc);
    }
}
