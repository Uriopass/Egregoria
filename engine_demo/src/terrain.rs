use engine::meshload::load_mesh;
use engine::terrain::TerrainRender as EngineTerrainRender;
use engine::{Context, FrameContext, InstancedMeshBuilder, MeshInstance};
use geom::{pack_height, vec2, Camera, Heightmap, HeightmapChunk, LinearColor, Vec3};

use crate::DemoElement;

const CSIZE: u32 = 512;
const CRESO: usize = 16;
const MAP_SIZE: usize = 50;

pub struct Terrain {
    terrain: EngineTerrainRender<CSIZE, CRESO>,
    heights: Heightmap<CRESO, { CSIZE }>,
    reload: bool,

    last_hitpos: Option<Vec3>,
    plane_hitpos: Option<Vec3>,
    hitmesh: InstancedMeshBuilder<false>,
}

impl DemoElement for Terrain {
    fn name(&self) -> &'static str {
        "Terrain"
    }

    #[allow(clippy::needless_range_loop)]
    fn init(ctx: &mut Context) -> Self {
        let gfx = &mut ctx.gfx;

        let hitmesh = load_mesh(gfx, "sphere.glb").unwrap();

        let mut h = Heightmap::new(MAP_SIZE as u16, MAP_SIZE as u16);

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let mut c = [[0; CRESO]; CRESO];
                for i in 0..CRESO {
                    for j in 0..CRESO {
                        c[i][j] = pack_height(
                            3000.0
                                * geom::fnoise::<6>(
                                    0.002 * vec2((x * CRESO + j) as f32, (y * CRESO + i) as f32),
                                )
                                .0
                                .powi(2),
                        );
                        //heights[y][x][i][j] =
                        //    (CSIZE / CRESO * i) as f32 + 0.5 * (CSIZE / CRESO * j) as f32;
                    }
                }
                h.set_chunk((x as u16, y as u16), HeightmapChunk::new(c));
            }
        }

        let mut terrain = EngineTerrainRender::new(gfx, MAP_SIZE as u32, MAP_SIZE as u32);

        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                terrain.update_chunk(
                    gfx,
                    (x as u32, y as u32),
                    h.get_chunk((x as u16, y as u16)).unwrap(),
                );
            }
        }

        terrain.invalidate_height_normals(&ctx.gfx);

        Self {
            terrain,
            heights: h,
            reload: false,
            last_hitpos: None,
            plane_hitpos: None,
            hitmesh: InstancedMeshBuilder::new(hitmesh),
        }
    }

    fn update(&mut self, ctx: &mut Context, cam: &Camera) {
        if self.reload {
            self.reload = false;
            self.terrain.invalidate_height_normals(&ctx.gfx);
        }

        self.last_hitpos = None;
        self.plane_hitpos = None;
        if let Some(unproj) = cam.unproj_ray(ctx.input.mouse.screen) {
            let p = geom::Plane { n: Vec3::Z, o: 0.0 };
            if let Some(mut v) = unproj.intersection_plane(&p) {
                v.z = self.heights.height(v.xy()).unwrap_or(0.0);
                self.plane_hitpos = Some(v);
            }

            if let Some((hitpos, _hitnormal)) = self.heights.raycast(unproj) {
                self.last_hitpos = Some(hitpos);
            }
        }
    }

    fn render(&mut self, fc: &mut FrameContext, cam: &Camera) {
        self.terrain.draw_terrain(cam, fc);

        self.hitmesh.instances.clear();
        if let Some(pos) = self.last_hitpos {
            self.hitmesh.instances.push(MeshInstance {
                pos,
                dir: Vec3::X * 20.0,
                tint: LinearColor::WHITE,
            });
        }
        if let Some(pos) = self.plane_hitpos {
            self.hitmesh.instances.push(MeshInstance {
                pos,
                dir: Vec3::X * 10.0,
                tint: LinearColor::RED,
            });
        }

        fc.draw(self.hitmesh.build(fc.gfx));
    }

    fn render_gui(&mut self, ui: &mut egui::Ui) {
        ui.indent("terrain", |ui| {
            if cfg!(debug_assertions) && ui.button("reload terrain").clicked() {
                self.reload = true;
            }
        });
    }
}
