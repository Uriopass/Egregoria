use crate::rendering::immediate::ImmediateDraw;
use crate::rendering::map_mesh::MapMeshHandler;
use crate::Context;
use common::FastMap;
use egregoria::map::{
    ChunkID, Lane, Map, ProjectFilter, ProjectKind, TrafficBehavior, CHUNK_RESOLUTION, CHUNK_SIZE,
};
use egregoria::Egregoria;
use geom::{vec3, Camera, Circle, InfiniteFrustrum, Intersect3, LinearColor, AABB3};
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::terrain::TerrainRender;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedMesh, InstancedMeshBuilder, MeshInstance, Water,
};

const CSIZE: usize = CHUNK_SIZE as usize;
const CRESO: usize = CHUNK_RESOLUTION;

/// Render the entire map including the terrain, trees, water etc
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
            terrain: TerrainRender::new(gfx, w, h, egregoria::config().border_col.into()),
            water: Water::new(gfx, (w * CHUNK_SIZE) as f32, (h * CHUNK_SIZE) as f32),
        }
    }

    fn render_lane_signals(n: &Lane, draw: &mut ImmediateDraw, time: u32) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + (dir_perp * -3.5 + dir * -1.0).z(0.02);

        if n.control.is_stop_sign() {
            draw.mesh("stop_sign.glb".to_string(), r_center, dir_perp.z(0.0));
            return;
        }

        let mesh = match n.control.get_behavior(time) {
            TrafficBehavior::RED | TrafficBehavior::STOP => "traffic_light_red.glb",
            TrafficBehavior::ORANGE => "traffic_light_orange.glb",
            TrafficBehavior::GREEN => "traffic_light_green.glb",
        };

        draw.mesh(mesh.to_string(), r_center, dir_perp.z(0.0));
    }

    fn signals_render(
        map: &Map,
        time: u32,
        cam: &Camera,
        frustrum: &InfiniteFrustrum,
        draw: &mut ImmediateDraw,
    ) {
        let pos = cam.pos;

        for n in map
            .spatial_map()
            .query(Circle::new(pos.xy(), 200.0), ProjectFilter::ROAD)
            .filter_map(|k| match k {
                ProjectKind::Road(id) => map.roads().get(id),
                _ => None,
            })
            .filter(|x| frustrum.intersects(&x.points.bbox().expand(x.width)))
            .flat_map(|r| r.lanes_iter())
            .map(|(id, _)| &map.lanes()[id])
        {
            Self::render_lane_signals(n, draw, time);
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

        self.terrain
            .update_borders(&mut ctx.gfx, &|p| ter.height(p));
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

    pub(crate) fn trees(
        &mut self,
        map: &Map,
        cam: &Camera,
        frustrum: &InfiniteFrustrum,
        ctx: &mut FrameContext<'_>,
    ) {
        self.build_trees(map, ctx);

        let camcenter = cam.pos.xy();

        for (cid, (_, meshes)) in self.trees_builders.iter() {
            let chunkcenter = vec3(
                (cid.0 * CHUNK_SIZE + CHUNK_SIZE / 2) as f32,
                (cid.1 * CHUNK_SIZE + CHUNK_SIZE / 2) as f32,
                0.0,
            );

            if !frustrum.intersects(&AABB3::centered(
                chunkcenter,
                vec3(5.0 + CHUNK_SIZE as f32, 5.0 + CHUNK_SIZE as f32, 100.0),
            )) || camcenter.distance(chunkcenter.xy()) > 5000.0
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
        frustrum: &InfiniteFrustrum,
        options: MapRenderOptions,
        draw: &mut ImmediateDraw,
        ctx: &mut FrameContext<'_>,
    ) {
        self.terrain.draw_terrain(cam, frustrum, ctx);

        self.trees(map, cam, frustrum, ctx);

        self.meshb.latest_mesh(map, options, ctx);

        Self::signals_render(map, time, cam, frustrum, draw);

        ctx.draw(self.water.clone());
    }
}
