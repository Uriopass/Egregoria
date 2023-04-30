use crate::rendering::immediate::ImmediateDraw;
use crate::rendering::map_mesh::MapMeshHandler;
use crate::Context;
use common::FastMap;
use egregoria::map::{
    ChunkID, Lane, LaneID, LaneKind, Map, ProjectFilter, ProjectKind, TrafficBehavior,
    CHUNK_RESOLUTION, CHUNK_SIZE,
};
use egregoria::Egregoria;
use flat_spatial::AABBGrid;
use geom::{vec3, Camera, Circle, InfiniteFrustrum, Intersect3, LinearColor, Vec3, AABB3, V3};
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::terrain::TerrainRender;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedMesh, InstancedMeshBuilder, LampLights, MeshInstance, Water,
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

        let grass = gfx.texture("assets/sprites/grass.jpg", "grass");

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
            terrain: TerrainRender::new(gfx, w, h, egregoria::config().border_col.into(), grass),
            water: Water::new(gfx, (w * CHUNK_SIZE) as f32, (h * CHUNK_SIZE) as f32),
        }
    }

    fn render_lane_signals(n: &Lane, draw: &mut ImmediateDraw, time: u32) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + (dir_perp * -4.5 + dir * -1.5).z(0.02);

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

    fn render_lanes(
        map: &Map,
        lanes: impl Iterator<Item = (LaneID, LaneKind)>,
        draw: &mut ImmediateDraw,
        time: u32,
    ) {
        let mut peek = lanes.peekable();

        while let Some((lane_id, kind)) = peek.next() {
            let peek = peek.peek();
            if let Some((_, peek_kind)) = peek {
                if peek_kind == &kind {
                    continue;
                }
            }
            let Some(lane) = map.lanes().get(lane_id) else { continue };
            Self::render_lane_signals(lane, draw, time);
        }
    }

    fn signals_render(
        map: &Map,
        time: u32,
        cam: &Camera,
        frustrum: &InfiniteFrustrum,
        draw: &mut ImmediateDraw,
    ) {
        let pos = cam.pos;

        for kind in map
            .spatial_map()
            .query(Circle::new(pos.xy(), 200.0), ProjectFilter::ROAD)
        {
            let ProjectKind::Road(id) = kind else { continue };
            let Some(r) = map.roads().get(id) else { continue };
            if !frustrum.intersects(&r.points.bbox().expand(r.width)) {
                continue;
            }

            Self::render_lanes(
                map,
                r.outgoing_lanes_from(r.dst).iter().copied(),
                draw,
                time,
            );
            Self::render_lanes(
                map,
                r.outgoing_lanes_from(r.src).iter().copied(),
                draw,
                time,
            );
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
    pub fn lampposts(&self, map: &Map, ctx: &mut FrameContext<'_>) {
        let lamps = &mut ctx.gfx.lamplights;
        if ctx.gfx.tick != 1 {
            return;
        }

        let mut by_chunk: AABBGrid<(), geom::AABB3> =
            flat_spatial::AABBGrid::new(LampLights::LIGHTCHUNK_SIZE as i32);

        let mut add_light = |p: Vec3| {
            by_chunk.insert(AABB3::centered(p, Vec3::splat(30.0)), ());
        };

        for x in map.roads().values() {
            let w = x.width * 0.5 - 5.0;
            for (point, dir) in x.points().equipoints_dir(45.0, true) {
                add_light(point + dir.perp_up() * w + 10.0 * V3::Z);
                add_light(point - dir.perp_up() * w + 10.0 * V3::Z);
            }
        }
        for i in map.intersections().values() {
            add_light(i.pos + 10.0 * V3::Z);
        }

        for (cell_idx, cell) in by_chunk.storage().cells.iter() {
            if cell.objs.is_empty() {
                continue;
            }
            if cell_idx.0 < 0 || cell_idx.1 < 0 {
                continue;
            }
            let mut i = 0;
            let lamp_poss: [Option<Vec3>; 4] = [(); 4].map(|_| {
                let v = cell
                    .objs
                    .get(i)
                    .and_then(|x| by_chunk.get(x.0))
                    .map(|x| x.aabb.center());
                i += 1;
                v
            });
            lamps.register_update((cell_idx.0 as u16, cell_idx.1 as u16), lamp_poss);
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
        self.lampposts(map, ctx);

        self.terrain.draw_terrain(cam, frustrum, ctx);

        self.trees(map, cam, frustrum, ctx);

        self.meshb.latest_mesh(map, options, ctx);

        Self::signals_render(map, time, cam, frustrum, draw);

        ctx.draw(self.water.clone());
    }
}
