use crate::DemoElement;
use engine::terrain::TerrainRender as EngineTerrainRender;
use engine::{Context, FrameContext};
use geom::{vec2, Camera, InfiniteFrustrum};

const CSIZE: usize = 512;
const CRESO: usize = 16;
const MAP_SIZE: usize = 50;

pub struct Terrain {
    terrain: EngineTerrainRender<CSIZE, CRESO>,
    _heights: Box<[[[[f32; CRESO]; CRESO]; MAP_SIZE]; MAP_SIZE]>,
    reload: bool,
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

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                for i in 0..CRESO {
                    for j in 0..CRESO {
                        heights[y][x][i][j] = 3000.0
                            * geom::fnoise::<6>(
                                0.002 * vec2((x * CRESO + j) as f32, (y * CRESO + i) as f32),
                            )
                            .0
                            .powi(2);
                        //heights[y][x][i][j] =
                        //    (CSIZE / CRESO * i) as f32 + 0.5 * (CSIZE / CRESO * j) as f32;
                    }
                }
            }
        }

        let grass = gfx.texture("assets/sprites/grass.jpg", "grass");

        let mut terrain = EngineTerrainRender::new(gfx, MAP_SIZE as u32, MAP_SIZE as u32, grass);

        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                terrain.update_chunk(gfx, (x as u32, y as u32), &heights[y][x]);
            }
        }

        terrain.invalidate_height_normals(&ctx.gfx);

        Self {
            terrain,
            _heights: heights,
            reload: false,
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        if self.reload {
            self.reload = false;
            self.terrain.invalidate_height_normals(&ctx.gfx);
        }
    }

    fn render(&mut self, fc: &mut FrameContext, cam: &Camera, frustrum: &InfiniteFrustrum) {
        self.terrain.draw_terrain(cam, frustrum, fc);
    }

    fn render_gui(&mut self, ui: &mut egui::Ui) {
        ui.indent("terrain", |ui| {
            if cfg!(debug_assertions) {
                if ui.button("reload terrain").clicked() {
                    self.reload = true;
                }
            }
        });
    }
}
