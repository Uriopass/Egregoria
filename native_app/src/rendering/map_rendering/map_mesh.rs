use crate::rendering::MapRenderOptions;
use common::FastMap;
use engine::earcut::earcut;
use engine::MeshBuilder;
use engine::{
    Drawable, FrameContext, GfxContext, InstancedMeshBuilder, Material, Mesh, MeshInstance,
    MeshVertex, MetallicRoughness, SpriteBatch, SpriteBatchBuilder, Tesselator,
};
use geom::{minmax, vec2, vec3, Color, LinearColor, PolyLine3, Polygon, Radians, Vec2, Vec3};
use prototypes::{FreightStationPrototype, GoodsCompanyPrototype, RenderAsset};
use simulation::map::{
    Building, BuildingKind, CanonicalPosition, Environment, Intersection, LaneKind, Lanes, LotKind,
    Map, MapSubscriber, ProjectFilter, ProjectKind, PylonPosition, Road, Roads, SubscriberChunkID,
    Turn, TurnKind, UpdateType, CROSSWALK_WIDTH, ROAD_Z_OFFSET,
};
use simulation::Simulation;
use std::ops::{Mul, Neg};
use std::sync::Arc;

/// This is the main struct that handles the map rendering.
/// It is responsible for generating the meshes and sprites for the map
/// That is, the mostly static things (roads, intersections, lights, buildings).
pub struct MapMeshHandler {
    builders: MapBuilders,
    cache: FastMap<SubscriberChunkID, CachedObj>,
    road_sub: MapSubscriber,
    building_sub: MapSubscriber,
}

#[derive(Default)]
struct CachedObj {
    road: Vec<Arc<Mesh>>,
    build: Vec<Arc<dyn Drawable>>,
    lots: Option<Mesh>,
    arrows: Option<SpriteBatch>,
}

impl CachedObj {
    fn is_empty(&self) -> bool {
        self.road.is_empty()
            && self.lots.is_none()
            && self.arrows.is_none()
            && self.build.is_empty()
    }
}

struct MapBuilders {
    buildsprites: FastMap<BuildingKind, SpriteBatchBuilder<false>>,
    buildmeshes: FastMap<BuildingKind, InstancedMeshBuilder<false>>,
    houses_mesh: MeshBuilder<false>,
    zonemeshes: FastMap<BuildingKind, (MeshBuilder<false>, InstancedMeshBuilder<false>, bool)>,
    arrow_builder: SpriteBatchBuilder<false>,
    crosswalk_builder: MeshBuilder<false>,
    mesh_map: MeshBuilder<false>,
    mesh_lots: MeshBuilder<false>,
}

impl MapMeshHandler {
    pub fn new(gfx: &mut GfxContext, sim: &Simulation) -> Self {
        let arrow_builder = SpriteBatchBuilder::from_path(gfx, "assets/sprites/arrow_one_way.png");

        let mut buildsprites = FastMap::default();
        let mut buildmeshes = FastMap::default();
        let mut zonemeshes = FastMap::default();

        for descr in GoodsCompanyPrototype::iter() {
            if descr.zone.is_some() {
                continue;
            }
            let RenderAsset::Sprite { path } = &descr.asset else {
                continue;
            };

            buildsprites.insert(
                BuildingKind::GoodsCompany(descr.id),
                SpriteBatchBuilder::new(&gfx.texture(path, "goods_company_tex"), gfx),
            );
        }

        for (asset, bkind) in GoodsCompanyPrototype::iter()
            .map(|descr| (&descr.asset, BuildingKind::GoodsCompany(descr.id)))
            .chain(
                FreightStationPrototype::iter()
                    .map(|descr| (&descr.asset, BuildingKind::RailFreightStation(descr.id))),
            )
            .chain([(
                &RenderAsset::Mesh {
                    path: "external_trading.glb".into(),
                },
                BuildingKind::ExternalTrading,
            )])
        {
            let RenderAsset::Mesh { path } = asset else {
                continue;
            };
            let m = match gfx.mesh(path) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to load mesh {}: {:?}", asset, e);
                    continue;
                }
            };

            buildmeshes.insert(bkind, InstancedMeshBuilder::new_ref(&m));
        }

        for descr in GoodsCompanyPrototype::iter() {
            let Some(ref z) = descr.zone else { continue };
            let floor = &z.floor;
            let filler = &z.filler;

            let floor_tex = gfx.texture(floor, "zone_floor_tex");
            let floor_mat = gfx.register_material(Material::new(
                gfx,
                &floor_tex,
                MetallicRoughness {
                    metallic: 0.0,
                    roughness: 1.0,
                    tex: None,
                },
                None,
            ));
            let floor_mesh = MeshBuilder::new(floor_mat);

            let m = match gfx.mesh(filler.as_ref()) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to load mesh for zone {}: {:?}", filler, e);
                    continue;
                }
            };

            let filler_mesh = InstancedMeshBuilder::new_ref(&m);

            zonemeshes.insert(
                BuildingKind::GoodsCompany(descr.id),
                (floor_mesh, filler_mesh, z.randomize_filler),
            );
        }

        let crosswalk_tex = gfx.texture("assets/sprites/crosswalk.png", "crosswalk");
        let crosswalk_mat = gfx.register_material(Material::new(
            gfx,
            &crosswalk_tex,
            MetallicRoughness {
                metallic: 0.0,
                roughness: 1.0,
                tex: None,
            },
            None,
        ));
        let houses_mat = gfx.register_material(Material::new(
            gfx,
            &gfx.palette(),
            MetallicRoughness {
                metallic: 0.0,
                roughness: 1.0,
                tex: None,
            },
            None,
        ));
        let builders = MapBuilders {
            arrow_builder,
            buildsprites,
            crosswalk_builder: MeshBuilder::new(crosswalk_mat),
            mesh_map: MeshBuilder::new(gfx.tess_material),
            houses_mesh: MeshBuilder::new(houses_mat),
            buildmeshes,
            zonemeshes,
            mesh_lots: MeshBuilder::new(gfx.tess_material),
        };

        Self {
            builders,
            cache: Default::default(),
            road_sub: sim.map().subscribe(UpdateType::Road),
            building_sub: sim.map().subscribe(UpdateType::Building),
        }
    }

    pub fn latest_mesh(
        &mut self,
        map: &Map,
        options: MapRenderOptions,
        ctx: &mut FrameContext<'_>,
    ) {
        profiling::scope!("draw map mesh");
        for chunk in self.road_sub.take_updated_chunks() {
            profiling::scope!("build road chunk");
            let b = &mut self.builders;
            b.map_mesh(map, chunk);

            let cached = self.cache.entry(chunk).or_default();

            cached.road.clear();
            cached.road.reserve(2);

            if let Some(mesh) = b.mesh_map.build(ctx.gfx) {
                cached.road.push(Arc::new(mesh));
            }
            if let Some(mesh) = b.crosswalk_builder.build(ctx.gfx) {
                cached.road.push(Arc::new(mesh));
            }

            cached.lots = b.mesh_lots.build(ctx.gfx);
            cached.arrows = b.arrow_builder.build(ctx.gfx);

            if cached.is_empty() {
                self.cache.remove(&chunk);
            }
        }

        for chunk in self.building_sub.take_updated_chunks() {
            profiling::scope!("build building chunk");

            let b = &mut self.builders;
            b.buildings_mesh(map, chunk);

            let cached = self.cache.entry(chunk).or_default();

            cached.build.clear();
            cached.build.reserve(4);

            let sprites = b
                .buildsprites
                .values_mut()
                .flat_map(|x| x.build(ctx.gfx))
                .collect::<Vec<_>>();

            if !sprites.is_empty() {
                cached.build.push(Arc::new(sprites));
            }

            let buildmeshes = b
                .buildmeshes
                .values_mut()
                .flat_map(|x| x.build(ctx.gfx))
                .collect::<Vec<_>>();

            if !buildmeshes.is_empty() {
                cached.build.push(Arc::new(buildmeshes));
            }

            if let Some(mesh) = b.houses_mesh.build(ctx.gfx) {
                cached.build.push(Arc::new(mesh));
            }

            let zonemeshes = b
                .zonemeshes
                .values_mut()
                .flat_map(|(a, b, _)| a.build(ctx.gfx).zip(b.build(ctx.gfx)))
                .collect::<Vec<_>>();
            if !zonemeshes.is_empty() {
                cached.build.push(Arc::new(zonemeshes));
            }

            if cached.is_empty() {
                self.cache.remove(&chunk);
            }
        }

        profiling::scope!("prepare map mesh");
        for v in self.cache.values() {
            ctx.draw(v.build.clone());
            ctx.draw(v.road.clone());
            if options.show_arrows {
                if let Some(ref x) = v.arrows {
                    ctx.draw(x.clone());
                }
            }
            if options.show_lots {
                if let Some(ref x) = v.lots {
                    ctx.draw(x.clone());
                }
            }
        }
    }
}

impl MapBuilders {
    fn arrows(arrow_builder: &mut SpriteBatchBuilder<false>, road: &Road, lanes: &Lanes) {
        let has_forward = road
            .outgoing_lanes_from(road.src)
            .iter()
            .filter(|(_, kind)| kind.needs_arrows())
            .count()
            > 0;
        let has_backward = road
            .outgoing_lanes_from(road.dst)
            .iter()
            .filter(|(_, kind)| kind.needs_arrows())
            .count()
            > 0;
        let is_two_way = has_forward && has_backward;

        if is_two_way {
            return;
        }

        let n_arrows = ((road.length() / 50.0) as i32).max(1);

        let fade =
            (road.length() - 5.0 - road.interface_from(road.src) - road.interface_from(road.dst))
                .mul(0.2)
                .clamp(0.0, 1.0);

        for (id, _) in road.lanes_iter().filter(|(_, kind)| kind.needs_arrows()) {
            let lane = &lanes[id];
            let l = lane.points.length();
            for i in 0..n_arrows {
                let (mid, dir) = lane
                    .points
                    .point_dir_along(l * (1.0 + i as f32) / (1.0 + n_arrows as f32));

                arrow_builder.push(
                    mid.up(0.03),
                    dir,
                    LinearColor::gray(0.3 + fade * 0.1),
                    (4.0, 4.0),
                );
            }
        }
    }

    fn crosswalks(crosswalk_builder: &mut MeshBuilder<false>, inter: &Intersection, lanes: &Lanes) {
        const WALKING_W: f32 = LaneKind::Walking.width();

        for turn in inter.turns() {
            let id = turn.id;

            if matches!(turn.kind, TurnKind::Crosswalk) {
                let from = lanes[id.src].get_inter_node_pos(inter.id).up(0.01);
                let to = lanes[id.dst].get_inter_node_pos(inter.id).up(0.01);

                let l = (to - from).mag();

                if l < WALKING_W {
                    continue;
                }

                let dir = (to - from) / l;
                let perp = dir.perp_up() * CROSSWALK_WIDTH * 0.5;
                let pos = from + dir * WALKING_W * 0.5;
                let height = l - WALKING_W;

                crosswalk_builder.extend_with(None, |vertices, add_index| {
                    let mk_v = |position: Vec3, uv: Vec2| MeshVertex {
                        position: position.into(),
                        uv: uv.into(),
                        normal: Vec3::Z,
                        color: [1.0; 4],
                        tangent: [0.0; 4],
                    };

                    vertices.push(mk_v(pos - perp, Vec2::ZERO));
                    vertices.push(mk_v(pos + perp, Vec2::ZERO));
                    vertices.push(mk_v(pos + perp + dir * height, Vec2::x(height)));
                    vertices.push(mk_v(pos - perp + dir * height, Vec2::x(height)));

                    add_index(0);
                    add_index(1);
                    add_index(2);

                    add_index(0);
                    add_index(2);
                    add_index(3);
                });
            }
        }
    }

    fn buildings_mesh(&mut self, map: &Map, chunk: SubscriberChunkID) {
        for v in self.buildsprites.values_mut() {
            v.clear();
        }
        for v in self.buildmeshes.values_mut() {
            v.instances.clear();
        }
        for v in self.zonemeshes.values_mut() {
            v.0.clear();
            v.1.instances.clear();
        }
        self.houses_mesh.clear();

        let buildings = &map.buildings();
        for building in map
            .spatial_map()
            .query(chunk.bbox(), ProjectFilter::BUILDING)
            .map(|p| {
                if let ProjectKind::Building(b) = p {
                    b
                } else {
                    unreachable!()
                }
            })
        {
            let building = &buildings[building];
            if SubscriberChunkID::new(building.canonical_position()) != chunk {
                continue;
            }
            self.zone_mesh(building);
            self.houses_mesh(building);

            if let Some(x) = self.buildsprites.get_mut(&building.kind) {
                let axis = building.obb.axis();
                let c = building.obb.center();
                let w = axis[0].mag();
                let d = axis[0] / w;
                let h = axis[1].mag();
                x.push(
                    c.z(building.height + 0.1),
                    d.z0(),
                    LinearColor::WHITE,
                    (w, h),
                );
            }

            if let Some(x) = self.buildmeshes.get_mut(&building.kind) {
                let pos = building.obb.center().z(building.height);
                let dir = building.obb.axis()[0].normalize().z0();

                x.instances.push(MeshInstance {
                    pos,
                    dir,
                    tint: LinearColor::WHITE,
                });
            }
        }
    }

    fn zone_mesh(&mut self, building: &Building) {
        let Some(bzone) = &building.zone else {
            return;
        };
        let Some((zone_mesh, filler, randomize)) = self.zonemeshes.get_mut(&building.kind) else {
            return;
        };
        let zone = &bzone.poly;
        let randomize = *randomize;

        let mut hull = building
            .mesh
            .faces
            .iter()
            .flat_map(|x| x.0.iter())
            .map(|x| x.xy())
            .collect::<Polygon>()
            .convex_hull();
        hull.simplify();
        hull.scale_from(hull.barycenter(), 1.8);

        let principal_axis = building.obb.axis()[0].normalize().rotated_by(bzone.filldir);

        let Some((mut min, mut max)) =
            minmax(zone.iter().map(|x| x.rotated_by(principal_axis.flipy())))
        else {
            return;
        };
        min = min.rotated_by(principal_axis);
        max = max.rotated_by(principal_axis);

        let secondary_axis = -principal_axis.perpendicular();

        let principal_dist = (max - min).dot(principal_axis).abs();
        let secondary_dist = (max - min).dot(secondary_axis).abs();

        for principal_offset in (0..=(principal_dist as i32)).step_by(4) {
            for secondary_offset in (0..=(secondary_dist as i32)).step_by(4) {
                let mut pos = min
                    + principal_axis * principal_offset as f32
                    + secondary_axis * secondary_offset as f32;
                if randomize {
                    pos = pos
                        + vec2(
                            common::rand::rand3(pos.x, pos.y, 10.0),
                            common::rand::rand3(pos.x, pos.y, 20.0),
                        ) * 2.0
                        - 1.0 * Vec2::XY;
                }

                if !zone.contains(pos) {
                    continue;
                }
                if zone.distance(pos) < 3.0 {
                    continue;
                }
                if hull.contains(pos) {
                    continue;
                }

                filler.instances.push(MeshInstance {
                    pos: pos.z(building.height),
                    dir: principal_axis.perpendicular().z0(),
                    tint: LinearColor::WHITE,
                });
            }
        }

        let avg = -zone.0.iter().sum::<Vec2>() / zone.len() as f32;

        zone_mesh.extend_with(None, |vertices, add_index| {
            for p in &zone.0 {
                vertices.push(MeshVertex {
                    position: p.z(building.height + 0.05).into(),
                    normal: Vec3::Z,
                    uv: ((*p + avg) * 0.05).into(),
                    color: [1.0; 4],
                    tangent: [0.0; 4],
                });
            }

            earcut(&zone.0, &[], |a, b, c| {
                add_index(a as u32);
                add_index(b as u32);
                add_index(c as u32);
            });
        })
    }

    fn houses_mesh(&mut self, building: &Building) {
        for (face, col) in &building.mesh.faces {
            self.houses_mesh.extend_with(None, |vertices, add_index| {
                let o = face[1];
                let u = unwrap_ret!((face[0] - o).try_normalize());
                let v = unwrap_ret!((face[2] - o).try_normalize());

                let mut nor = u.cross(v);

                let mut reverse = false;

                if nor.z < 0.0 {
                    reverse = true;
                    nor = -nor;
                }

                let mut projected = Polygon(Vec::with_capacity(face.len()));
                for &p in face {
                    let off = p - o;
                    projected.0.push(vec2(off.dot(u), off.dot(v)));

                    vertices.push(MeshVertex {
                        position: p.into(),
                        normal: nor,
                        uv: [0.0; 2],
                        color: col.into(),
                        tangent: [0.0; 4],
                    })
                }

                projected.simplify();

                earcut(&projected.0, &[], |mut a, b, mut c| {
                    if reverse {
                        std::mem::swap(&mut a, &mut c);
                    }
                    add_index(a as u32);
                    add_index(b as u32);
                    add_index(c as u32);
                })
            });
        }
    }

    fn draw_rail(tess: &mut Tesselator, cut: &PolyLine3, off: f32, _limits: bool) {
        tess.set_color(Color::gray(0.5));
        tess.draw_polyline_full(
            cut.as_slice().iter().map(|v| vec3(v.x, v.y, v.z + 0.02)),
            unwrap_ret!(cut.first_dir()).xy(),
            unwrap_ret!(cut.last_dir()).xy(),
            0.1,
            off + 0.6,
        );
        tess.draw_polyline_full(
            cut.as_slice().iter().map(|v| vec3(v.x, v.y, v.z + 0.02)),
            unwrap_ret!(cut.first_dir()).xy(),
            unwrap_ret!(cut.last_dir()).xy(),
            0.1,
            off - 0.6,
        );
        //for (v, dir) in cut.equipoints_dir(1.0, !limits) {
        //    let up = vec3(v.x, v.y, v.z + 0.04);
        //    tess.draw_polyline_full(
        //        [up, up + dir * 0.1].into_iter(),
        //        dir.xy(),
        //        dir.xy(),
        //        2.0,
        //        off,
        //    );
        //}
    }

    fn map_mesh(&mut self, map: &Map, chunk: SubscriberChunkID) {
        self.arrow_builder.clear();
        self.crosswalk_builder.clear();
        self.mesh_map.clear();
        self.mesh_lots.clear();

        let mut tess_map = self.mesh_map.mk_tess();
        let mut tess_lots = self.mesh_lots.mk_tess();

        let low_col: LinearColor = simulation::colors().road_low_col.into();
        let mid_col: LinearColor = simulation::colors().road_mid_col.into();
        let hig_col: LinearColor = simulation::colors().road_hig_col.into();
        let line_col: LinearColor = simulation::colors().road_line_col.into();

        let objs = map.spatial_map().query(
            chunk.bbox(),
            ProjectFilter::ROAD | ProjectFilter::LOT | ProjectFilter::INTER,
        );

        let mut chunk_roads = Vec::new();
        let mut chunk_lots = Vec::new();
        let mut chunk_inters = Vec::new();

        for obj in objs {
            if SubscriberChunkID::new(obj.canonical_position(map)) != chunk {
                continue;
            }
            match obj {
                ProjectKind::Road(road) => chunk_roads.push(road),
                ProjectKind::Lot(lot) => chunk_lots.push(lot),
                ProjectKind::Intersection(inter) => chunk_inters.push(inter),
                _ => {}
            }
        }

        let inters = map.intersections();
        let lanes = map.lanes();
        let roads = map.roads();
        let lots = map.lots();
        let env = &map.environment;

        for road in chunk_roads {
            let road = &roads[road];

            Self::arrows(&mut self.arrow_builder, road, lanes);

            let cut = road.interfaced_points();
            let first_dir = unwrap_cont!(cut.first_dir());
            let last_dir = unwrap_cont!(cut.last_dir());

            road_pylons(&mut tess_map, env, road);

            tess_map.normal.z = -1.0;
            tess_map.draw_polyline_full(
                cut.iter().map(|x| x.up(-0.3)),
                first_dir.xy(),
                last_dir.xy(),
                road.width,
                0.0,
            );
            tess_map.normal.z = 1.0;

            let draw_off = |tess: &mut Tesselator, col: LinearColor, w, off| {
                tess.set_color(col);
                tess.draw_polyline_full(
                    cut.as_slice().iter().copied(),
                    first_dir.xy(),
                    last_dir.xy(),
                    w,
                    off,
                );
            };

            let mut start = true;
            for l in road.lanes_iter().flat_map(|(l, _)| lanes.get(l)) {
                if l.kind.is_rail() {
                    let off = l.dist_from_bottom - road.width * 0.5 + LaneKind::Rail.width() * 0.5;
                    draw_off(&mut tess_map, mid_col, LaneKind::Rail.width(), off);
                    Self::draw_rail(&mut tess_map, cut, off, true);
                    start = true;
                    continue;
                }
                if start {
                    draw_off(
                        &mut tess_map,
                        line_col,
                        0.25,
                        l.dist_from_bottom - road.width * 0.5,
                    );
                    start = false;
                }
                draw_off(
                    &mut tess_map,
                    match l.kind {
                        LaneKind::Walking => hig_col,
                        LaneKind::Parking => low_col,
                        _ => mid_col,
                    },
                    l.kind.width() - 0.25,
                    l.dist_from_bottom - road.width * 0.5 + l.kind.width() * 0.5,
                );
                draw_off(
                    &mut tess_map,
                    line_col,
                    0.25,
                    l.dist_from_bottom - road.width * 0.5 + l.kind.width(),
                );
            }
        }

        // Intersections
        let mut p = Vec::with_capacity(8);
        let mut ppoly = unsafe { PolyLine3::new_unchecked(vec![]) };
        for inter in chunk_inters {
            let inter = &inters[inter];

            let interpos = inter.pos.up(ROAD_Z_OFFSET);

            if inter.roads.is_empty() {
                tess_map.set_color(line_col);
                tess_map.draw_circle(interpos, 5.5);

                tess_map.set_color(mid_col);
                tess_map.draw_circle(interpos, 5.0);
                continue;
            }

            Self::crosswalks(&mut self.crosswalk_builder, inter, lanes);

            inter_pylon(&mut tess_map, env, inter, roads);
            intersection_mesh(&mut tess_map, &hig_col, inter, roads);

            // Walking corners
            for turn in inter
                .turns()
                .filter(|turn| matches!(turn.kind, TurnKind::WalkingCorner))
            {
                tess_map.set_color(line_col);
                let id = turn.id;

                let w = lanes[id.src].kind.width();

                let first_dir = -lanes[id.src].orientation_from(id.parent);
                let last_dir = lanes[id.dst].orientation_from(id.parent);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess_map.draw_polyline_full(p.iter().copied(), first_dir, last_dir, 0.25, w * 0.5);
                tess_map.draw_polyline_full(p.iter().copied(), first_dir, last_dir, 0.25, -w * 0.5);

                tess_map.set_color(hig_col);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess_map.draw_polyline_with_dir(&p, first_dir, last_dir, w - 0.25);
            }

            // Rail turns
            for turn in inter
                .turns()
                .filter(|turn| matches!(turn.kind, TurnKind::Rail))
            {
                ppoly.clear_extend(turn.points.as_slice());
                Self::draw_rail(&mut tess_map, &ppoly, 0.0, false);
            }
        }

        // Lots
        for lot in chunk_lots {
            let lot = &lots[lot];
            let col = match lot.kind {
                LotKind::Unassigned => simulation::colors().lot_unassigned_col,
                LotKind::Residential => simulation::colors().lot_residential_col,
            };
            tess_lots.set_color(col);
            tess_lots.draw_filled_polygon(&lot.shape.corners, lot.height + 0.28);
        }
    }
}

fn add_polyon(
    mut tess: &mut Tesselator,
    w: f32,
    PylonPosition {
        terrain_height,
        pos,
        dir,
    }: PylonPosition,
) {
    let color = LinearColor::from(simulation::colors().road_pylon_col);
    let color: [f32; 4] = color.into();

    let up = pos.up(-0.2);
    let down = pos.xy().z(terrain_height - 20.0);
    let dirp = dir.perp_up();
    let d2 = dir.xy().z0();
    let d2p = d2.perp_up();
    let d2 = d2 * w * 0.5;
    let d2p = d2p * w * 0.5;
    let dir = dir * w * 0.5;
    let dirp = dirp * w * 0.5;
    // down rect
    // 2 --- 1 -> dir
    // |     |
    // |     |
    // 3-----0
    // | dirp
    // v

    // up rect
    // 6 --- 5
    // |     |
    // |     |
    // 7-----4
    let verts = [
        down + d2 + d2p, // 0
        down + d2 - d2p, // 1
        down - d2 - d2p, // 2
        down - d2 + d2p, // 3
        up + dir + dirp, // 4
        up + dir - dirp, // 5
        up - dir - dirp, // 6
        up - dir + dirp, // 7
    ];

    let mr = &mut tess;
    let mut quad = move |a, b, c, d, nor| {
        mr.extend_with(move |vertices, add_idx| {
            let mut pvert = move |p: Vec3, normal: Vec3| {
                vertices.push(MeshVertex {
                    position: p.into(),
                    normal,
                    uv: [0.0; 2],
                    color,
                    tangent: [0.0; 4],
                })
            };

            pvert(verts[a], nor);
            pvert(verts[b], nor);
            pvert(verts[c], nor);
            pvert(verts[d], nor);

            add_idx(0);
            add_idx(1);
            add_idx(2);

            add_idx(1);
            add_idx(3);
            add_idx(2);
        });
    };
    quad(0, 1, 4, 5, d2);
    quad(1, 2, 5, 6, -d2p);
    quad(2, 3, 6, 7, -d2);
    quad(3, 0, 7, 4, d2p);
}

fn road_pylons(meshb: &mut Tesselator, env: &Environment, road: &Road) {
    for pylon in Road::pylons_positions(road.interfaced_points(), env) {
        add_polyon(meshb, road.width * 0.5, pylon);
    }
}

fn inter_pylon(tess: &mut Tesselator, env: &Environment, inter: &Intersection, roads: &Roads) {
    let interpos = inter.pos.up(ROAD_Z_OFFSET);

    let h = unwrap_ret!(env.true_height(inter.pos.xy()));
    if (h - interpos.z).abs() <= 2.0 {
        return;
    }

    let mut maxw = 3.0f32;
    let mut avgp = Vec3::ZERO;

    for &road in &inter.roads {
        let r = &roads[road];
        maxw = maxw.max(r.width * 0.5);
        avgp += r.interface_point(inter.id);
    }
    if !inter.roads.is_empty() {
        avgp /= inter.roads.len() as f32;
    } else {
        avgp = interpos;
    }

    add_polyon(
        tess,
        maxw,
        PylonPosition {
            terrain_height: h,
            pos: avgp,
            dir: Vec3::X,
        },
    );
}

fn intersection_mesh(
    tess: &mut Tesselator,
    center_col: &LinearColor,
    inter: &Intersection,
    roads: &Roads,
) {
    let interpos = inter.pos.up(ROAD_Z_OFFSET);
    let id = inter.id;

    let getw = |road: &Road| {
        if road.sidewalks(id).outgoing.is_some() {
            road.width * 0.5 - LaneKind::Walking.width()
        } else {
            road.width * 0.5
        }
    };

    let mut polygon = Polygon::default();

    for (i, &road) in inter.roads.iter().enumerate() {
        #[allow(clippy::indexing_slicing)]
        let road = &roads[road];

        #[allow(clippy::indexing_slicing)]
        let next_road = &roads[inter.roads[(i + 1) % inter.roads.len()]];

        let ip = road.interfaced_points();

        let firstp;
        let firstdir;
        if road.dst == inter.id {
            firstp = ip.last();
            firstdir = ip.last_dir().map(Vec3::neg);
        } else {
            firstp = ip.first();
            firstdir = ip.first_dir();
        }

        let src_orient = -unwrap_cont!(firstdir).xy();

        let left = firstp.xy() + src_orient.perpendicular() * getw(road);

        let ip = next_road.interfaced_points();

        let firstp;
        let firstdir;
        if next_road.dst == inter.id {
            firstp = ip.last();
            firstdir = ip.last_dir().map(Vec3::neg);
        } else {
            firstp = ip.first();
            firstdir = ip.first_dir();
        }

        let dst_orient = unwrap_cont!(firstdir).xy();
        let next_right = firstp.xy() + dst_orient.perpendicular() * getw(next_road);

        if inter.is_roundabout() {
            if let Some(rp) = inter.turn_policy.roundabout {
                let center = interpos.xy();

                let ang = (left - center)
                    .normalize()
                    .angle((next_right - center).normalize())
                    .abs();
                if ang >= Radians::from_deg(21.0).0 {
                    polygon.extend(Turn::gen_roundabout(
                        left.z(0.0),
                        next_right.z(0.0),
                        src_orient,
                        dst_orient,
                        rp.radius + 3.0,
                        center,
                    ));

                    tess.set_color(center_col);
                    tess.draw_circle(center.z(interpos.z + 0.01), rp.radius * 0.5);

                    continue;
                }
            }
        }

        let spline = Turn::spline(left, next_right, src_orient, dst_orient);

        polygon.extend(spline.smart_points(1.0, 0.0, 1.0));
    }

    polygon.simplify();

    let col = LinearColor::from(simulation::colors().road_mid_col).into();
    tess.extend_with(move |vertices, add_idx| {
        vertices.extend(polygon.iter().map(|pos| MeshVertex {
            position: pos.z(interpos.z - 0.001).into(),
            normal: Vec3::Z,
            uv: [0.0; 2],
            color: col,
            tangent: [0.0; 4],
        }));
        earcut(&polygon.0, &[], |a, b, c| {
            add_idx(a as u32);
            add_idx(b as u32);
            add_idx(c as u32);
            add_idx(c as u32);
            add_idx(b as u32);
            add_idx(a as u32);
        });
    });
}
