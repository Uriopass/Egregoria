use crate::rendering::immediate::ImmediateDraw;
use crate::rendering::map_mesh::MapMeshHandler;
use crate::Context;
use common::{FastMap, FastSet};
use egregoria::map::{
    chunk_id, Chunk, ChunkID, Lane, LaneID, LaneKind, Map, MapSubscriber, ProjectFilter,
    ProjectKind, TrafficBehavior, UpdateType, CHUNK_RESOLUTION, CHUNK_SIZE,
};
use egregoria::Egregoria;
use flat_spatial::AABBGrid;
use geom::{
    vec3, vec4, Camera, Circle, InfiniteFrustrum, Intersect3, LinearColor, Matrix4, Vec3, AABB3, V3,
};
use std::ops::Mul;
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::terrain::TerrainRender;
use wgpu_engine::wgpu::RenderPass;
use wgpu_engine::{
    Drawable, FrameContext, GfxContext, InstancedMesh, InstancedMeshBuilder, LampLights,
    LightChunkID, MeshInstance, Water,
};

const CSIZE: usize = CHUNK_SIZE as usize;
const CRESO: usize = CHUNK_RESOLUTION;

/// Render the entire map including the terrain, trees, water etc
pub struct MapRenderer {
    pub meshb: MapMeshHandler,

    terrain: TerrainRender<CSIZE, CRESO>,
    terrain_sub: MapSubscriber,

    tree_builder: InstancedMeshBuilder<false>,
    trees_cache: FastMap<ChunkID, InstancedMesh>,
    tree_sub: MapSubscriber,

    water: Water,

    lamp_memory: FastMap<LightChunkID, Vec<Vec3>>,
    lamp_road_memory: FastMap<ChunkID, Vec<(LightChunkID, Vec3)>>,
    lamp_sub: MapSubscriber,
}

pub struct MapRenderOptions {
    pub show_arrows: bool,
}

impl MapRenderer {
    pub fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let mesh = load_mesh(gfx, "pine.glb").expect("could not load pine");

        let w = goria.map().terrain.width;
        let h = goria.map().terrain.height;

        let grass = gfx.texture("assets/sprites/grass.jpg", "grass");

        let terrain = TerrainRender::new(gfx, w, h, egregoria::config().border_col.into(), grass);

        /*
        let ter = &goria.map().terrain;
        let minchunk = *ter.chunks.keys().min().unwrap();
        let maxchunk = *ter.chunks.keys().max().unwrap();
        terrain.update_borders(minchunk, maxchunk, gfx, &|p| ter.height(p));
         */

        defer!(log::info!("finished init of road render"));
        MapRenderer {
            meshb: MapMeshHandler::new(gfx, goria),
            tree_builder: InstancedMeshBuilder::new(mesh.clone()),
            trees_cache: FastMap::default(),
            tree_sub: goria.map().subscribe(UpdateType::Terrain),
            terrain,
            water: Water::new(gfx, (w * CHUNK_SIZE) as f32, (h * CHUNK_SIZE) as f32),
            terrain_sub: goria.map().subscribe(UpdateType::Terrain),
            lamp_sub: goria.map().subscribe(UpdateType::Road),
            lamp_memory: Default::default(),
            lamp_road_memory: Default::default(),
        }
    }

    fn render_lane_signals(n: &Lane, draw: &mut ImmediateDraw, time: u32) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + (dir_perp * -5.2 + dir * -1.5).z(0.02);

        if n.control.is_stop_sign() {
            draw.mesh("stop_sign.glb", r_center, dir_perp.z(0.0));
            return;
        }

        let mesh = match n.control.get_behavior(time) {
            TrafficBehavior::RED | TrafficBehavior::STOP => "traffic_light_red.glb",
            TrafficBehavior::ORANGE => "traffic_light_orange.glb",
            TrafficBehavior::GREEN => "traffic_light_green.glb",
        };

        draw.mesh(mesh, r_center, dir_perp.z(0.0));
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

            if !r.lanes_iter().all(|(_, kind)| kind.is_rail()) {
                let has_sidewalks = r.has_sidewalks();
                let offset = if has_sidewalks {
                    LaneKind::Walking.width()
                } else {
                    0.0
                };
                let w = r.width * 0.5 - offset;
                for (point, dir) in r.points().equipoints_dir(45.0, true) {
                    draw.mesh("streetlamp.glb", point - dir.perp_up() * w, dir.perp_up());
                }
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

    pub fn terrain_update(&mut self, ctx: &mut Context, goria: &Egregoria) {
        let map = goria.map();
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

    pub fn build_trees(&mut self, map: &Map, ctx: &mut FrameContext<'_>) {
        for chunkid in self.tree_sub.take_updated_chunks() {
            let chunk = if let Some(x) = map.terrain.chunks.get(&chunkid) {
                x
            } else {
                continue;
            };

            self.tree_builder.instances.clear();

            for t in &chunk.trees {
                self.tree_builder.instances.push(MeshInstance {
                    pos: t.pos.z(map.terrain.height(t.pos).unwrap_or_default()),
                    dir: t.dir.z0() * t.size * 0.2,
                    tint: ((1.0 - t.size * 0.05) * t.col * LinearColor::WHITE).a(1.0),
                });
            }

            if let Some(m) = self.tree_builder.build(ctx.gfx) {
                self.trees_cache.insert(chunkid, m);
            } else {
                self.trees_cache.remove(&chunkid);
            }
        }
    }

    pub fn trees(
        &mut self,
        map: &Map,
        cam: &Camera,
        frustrum: &InfiniteFrustrum,
        ctx: &mut FrameContext<'_>,
    ) {
        self.build_trees(map, ctx);

        let camcenter = cam.pos.xy();

        struct TreeMesh(InstancedMesh, Vec3);

        impl Drawable for TreeMesh {
            fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
                self.0.draw(gfx, rp);
            }
            fn draw_depth<'a>(
                &'a self,
                gfx: &'a GfxContext,
                rp: &mut RenderPass<'a>,
                shadow_cascade: Option<&Matrix4>,
                proj: &'a wgpu_engine::wgpu::BindGroup,
            ) {
                if let Some(v) = shadow_cascade {
                    let pos = v.mul(self.1.w(1.0));

                    let margin = v.mul(vec4(
                        CHUNK_SIZE as f32 * 1.5,
                        CHUNK_SIZE as f32 * 1.5,
                        100.0,
                        0.0,
                    )) * pos.w;

                    if pos.x.abs() > pos.w + margin.x.abs()
                        || pos.y.abs() > pos.w + margin.y.abs()
                        || pos.z < -margin.z.abs()
                        || pos.z > pos.w + margin.z.abs()
                    {
                        return;
                    }
                }
                self.0.draw_depth(gfx, rp, shadow_cascade, proj);
            }
        }

        for (cid, mesh) in self.trees_cache.iter() {
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

            ctx.draw(TreeMesh(mesh.clone(), chunkcenter));
        }
    }

    pub fn lampposts(&mut self, map: &Map, ctx: &mut FrameContext<'_>) {
        profiling::scope!("lampposts");

        let mut to_reupload: FastSet<LightChunkID> = Default::default();
        for chunk in self.lamp_sub.take_updated_chunks() {
            let lamp_chunk_memory = self.lamp_road_memory.entry(chunk).or_default();
            for (chunk_id, lamp) in lamp_chunk_memory.drain(..) {
                let Some(lamps) = self.lamp_memory.get_mut(&chunk_id) else { continue };
                let Some(idx) = lamps.iter().position(|x| *x == lamp) else { continue };
                to_reupload.insert(chunk_id);
                lamps.swap_remove(idx);
            }

            let mut by_chunk: AABBGrid<(), AABB3> =
                AABBGrid::new(LampLights::LIGHTCHUNK_SIZE as i32);

            let mut add_light = |p: Vec3| {
                by_chunk.insert(AABB3::centered(p, Vec3::splat(64.0)), ());
            };

            let mut chunk_roads = vec![];
            let mut chunk_inter = vec![];

            map.spatial_map()
                .query(
                    Chunk::rect(chunk),
                    ProjectFilter::ROAD | ProjectFilter::INTER,
                )
                .for_each(|proj| {
                    if chunk_id(proj.canonical_position(map)) != chunk {
                        return;
                    }
                    match proj {
                        ProjectKind::Road(rid) => chunk_roads.push(rid),
                        ProjectKind::Inter(iid) => chunk_inter.push(iid),
                        _ => unreachable!(),
                    }
                });

            let roads = map.roads();
            let inters = map.intersections();

            for road in chunk_roads {
                let road = &roads[road];
                if road.lanes_iter().all(|(_, kind)| kind.is_rail()) {
                    continue;
                }
                for (point, _) in road.points().equipoints_dir(45.0, true) {
                    add_light(point + 8.0 * V3::Z);
                }
            }
            for i in chunk_inter {
                let i = &inters[i];
                if i.roads
                    .iter()
                    .filter_map(|&rid| map.roads().get(rid))
                    .all(|r| r.lanes_iter().all(|(_, kind)| kind.is_rail()))
                {
                    continue;
                }

                add_light(i.pos + 8.0 * V3::Z);
            }

            for (cell_idx, cell) in by_chunk.storage().cells.iter() {
                if cell.objs.is_empty() {
                    continue;
                }
                if cell_idx.0 < 0 || cell_idx.1 < 0 {
                    continue;
                }

                let lamp_poss = cell
                    .objs
                    .iter()
                    .filter_map(|x| by_chunk.get(x.0))
                    .map(|x| x.aabb.center());

                let lchunk_id = (cell_idx.0 as u16, cell_idx.1 as u16);
                let lamp_light_memory = self.lamp_memory.entry(lchunk_id).or_default();

                for v in lamp_poss {
                    lamp_light_memory.push(v);
                    lamp_chunk_memory.push((lchunk_id, v));
                }
                to_reupload.insert(lchunk_id);
            }
        }

        for chunk in to_reupload {
            let lamps = &self.lamp_memory[&chunk];
            ctx.gfx
                .lamplights
                .register_update(chunk, lamps.iter().copied());
        }
    }

    #[profiling::function]
    pub fn render(
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
