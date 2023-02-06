use crate::rendering::map_mesh::MapMeshHandler;
use crate::Context;
use common::FastMap;
use egregoria::map::{
    ChunkID, Lane, Map, ProjectFilter, ProjectKind, TrafficBehavior, CHUNK_RESOLUTION, CHUNK_SIZE,
};
use egregoria::Egregoria;
use geom::{vec3, Camera, Color, LinearColor};
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::terrain::TerrainRender;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedMesh, InstancedMeshBuilder, MeshInstance, Tesselator, Water,
};

const CSIZE: usize = CHUNK_SIZE as usize;
const CRESO: usize = CHUNK_RESOLUTION;

pub(crate) struct MapRenderer {
    pub(crate) meshb: MapMeshHandler,

    terrain: TerrainRender<CSIZE, CRESO>,

    #[allow(clippy::type_complexity)]
    trees_builders: FastMap<ChunkID, (InstancedMeshBuilder, Option<(Option<InstancedMesh>, u32)>)>,
    pub(crate) terrain_dirt_id: u32,
    water: Water,
}

pub struct MapRenderOptions {
    pub(crate) show_arrows: bool,
}

impl MapRenderer {
    pub(crate) fn reset(&mut self) {
        self.terrain_dirt_id = 0;
        self.terrain.reset();

        self.meshb.map_dirt_id = 0;
        for v in self.trees_builders.values_mut() {
            v.1 = None;
        }
    }
}

impl MapRenderer {
    pub(crate) fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let mesh = load_mesh(gfx, "pine.glb").expect("could not load pine");

        let w = goria.map().terrain.width;
        let h = goria.map().terrain.height;

        defer!(log::info!("finished init of road render"));
        MapRenderer {
            meshb: MapMeshHandler::new(gfx, goria),
            trees_builders: goria
                .map()
                .terrain
                .chunks
                .keys()
                .map(|id| (*id, (InstancedMeshBuilder::new(mesh.clone()), None)))
                .collect(),
            terrain_dirt_id: 0,
            terrain: TerrainRender::new(gfx, w, h),
            water: Water::new(gfx, (w * CHUNK_SIZE) as f32, (h * CHUNK_SIZE) as f32),
        }
    }

    fn render_lane_signals(n: &Lane, sr: &mut Tesselator, time: u32) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + (dir_perp * -3.5 + dir * -1.0).z(0.02);

        // Stop sign
        if n.control.is_stop_sign() {
            sr.set_color(LinearColor::WHITE);
            sr.draw_regular_polygon(r_center, 0.5, 8, std::f32::consts::FRAC_PI_8);

            sr.set_color(LinearColor::RED);
            sr.draw_regular_polygon(r_center, 0.4, 8, std::f32::consts::FRAC_PI_8);
            return;
        }

        // Traffic light
        let size = 0.5; // light size

        sr.color = Color::gray(0.2).into();
        sr.draw_rect_cos_sin(r_center, size + 0.1, size * 3.0 + 0.1, dir);

        for i in -1..2 {
            sr.draw_circle(r_center + i as f32 * dir_perp.z0() * size, size * 0.5);
        }
        sr.set_color(match n.control.get_behavior(time) {
            TrafficBehavior::RED | TrafficBehavior::STOP => LinearColor::RED,
            TrafficBehavior::ORANGE => LinearColor::ORANGE,
            TrafficBehavior::GREEN => LinearColor::GREEN,
        });

        let offset = match n.control.get_behavior(time) {
            TrafficBehavior::RED => -size,
            TrafficBehavior::ORANGE => 0.0,
            TrafficBehavior::GREEN => size,
            TrafficBehavior::STOP => unreachable!(),
        };

        sr.draw_circle(r_center + offset * dir_perp.z0(), size * 0.5);
    }

    fn signals_render(map: &Map, time: u32, sr: &mut Tesselator) {
        match sr.cull_rect {
            Some(rect) => {
                if rect.w().max(rect.h()) > 1500.0 {
                    return;
                }
                for n in map
                    .spatial_map()
                    .query(rect, ProjectFilter::ROAD)
                    .filter_map(|k| match k {
                        ProjectKind::Road(id) => Some(id),
                        _ => None,
                    })
                    .flat_map(|id| map.roads()[id].lanes_iter())
                    .map(|(id, _)| &map.lanes()[id])
                {
                    Self::render_lane_signals(n, sr, time);
                }
            }
            None => {
                for n in map.lanes().values() {
                    Self::render_lane_signals(n, sr, time);
                }
            }
        }
    }

    pub(crate) fn terrain_update(&mut self, ctx: &mut Context, goria: &Egregoria) {
        let map = goria.map();
        let ter = &map.terrain;
        if ter.dirt_id.0 == self.terrain.dirt_id {
            return;
        }

        let mut update_count = 0;
        for &cell in ter.chunks.keys() {
            let chunk = unwrap_retlog!(ter.chunks.get(&cell), "trying to update nonexistent chunk");

            if self
                .terrain
                .update_chunk(&mut ctx.gfx, chunk.dirt_id.0, cell, &chunk.heights)
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
        if update_count == 0 {
            self.terrain.dirt_id = ter.dirt_id.0;
        }

        self.terrain.update_borders(&ctx.gfx, &|p| ter.height(p));
    }

    pub(crate) fn build_trees(&mut self, map: &Map, ctx: &mut FrameContext<'_>) {
        if map.terrain.dirt_id.0 == self.terrain_dirt_id {
            return;
        }
        self.terrain_dirt_id = map.terrain.dirt_id.0;

        for (chunkid, (builder, mesh_dirt)) in &mut self.trees_builders {
            let chunk = if let Some(x) = map.terrain.chunks.get(chunkid) {
                x
            } else {
                continue;
            };

            if let Some((_, dirt)) = mesh_dirt {
                if *dirt == chunk.dirt_id.0 {
                    continue;
                }
            }

            builder.instances.clear();

            for t in &chunk.trees {
                builder.instances.push(MeshInstance {
                    pos: t.pos.z(map.terrain.height(t.pos).unwrap_or_default()),
                    dir: t.dir.z0() * t.size * 0.2,
                    tint: ((1.0 - t.size * 0.05) * t.col * LinearColor::WHITE).a(1.0),
                });
            }

            *mesh_dirt = Some((builder.build(ctx.gfx), chunk.dirt_id.0));
        }
    }

    pub(crate) fn trees(&mut self, map: &Map, cam: &Camera, ctx: &mut FrameContext<'_>) {
        self.build_trees(map, ctx);

        let camcenter = cam.pos.xy();
        let eye = cam.eye();
        let dir = -cam.dir();

        for (cid, (_, meshes)) in self.trees_builders.iter() {
            let chunkcenter = vec3(
                (cid.0 * CHUNK_SIZE + CHUNK_SIZE / 2) as f32,
                (cid.1 * CHUNK_SIZE + CHUNK_SIZE / 2) as f32,
                0.0,
            );

            if ((chunkcenter - eye).dot(dir) < 0.0
                || chunkcenter.xy().distance(camcenter) > 10000.0)
                && !chunkcenter.xy().is_close(eye.xy(), CHUNK_SIZE as f32)
            {
                continue;
            }

            if let Some((Some(mesh), _)) = meshes {
                ctx.draw(mesh.clone());
            }
        }
    }

    #[profiling::function]
    pub(crate) fn render(
        &mut self,
        map: &Map,
        time: u32,
        cam: &Camera,
        options: MapRenderOptions,
        tess: &mut Tesselator,
        ctx: &mut FrameContext<'_>,
    ) {
        self.terrain.draw_terrain(cam, ctx);

        self.trees(map, cam, ctx);

        self.meshb.latest_mesh(map, options, ctx);

        Self::signals_render(map, time, tess);

        ctx.draw(self.water.clone());
    }
}
