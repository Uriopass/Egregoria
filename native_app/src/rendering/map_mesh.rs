use common::FastMap;
use egregoria::map::{
    BuildingKind, Intersection, LaneKind, LotKind, Map, PylonPosition, Road, Roads, Terrain,
    TurnKind, CROSSWALK_WIDTH,
};
use egregoria::souls::goods_company::GoodsCompanyRegistry;
use egregoria::Egregoria;
use geom::{minmax, vec2, vec3, Color, LinearColor, PolyLine3, Polygon, Spline, Vec2, Vec3};
use std::ops::{Mul, Neg};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wgpu_engine::earcut::earcut;
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::wgpu::RenderPass;
use wgpu_engine::{
    Drawable, GfxContext, InstancedMesh, InstancedMeshBuilder, Mesh, MeshBuilder, MeshInstance,
    MeshVertex, MultiSpriteBatch, SpriteBatch, SpriteBatchBuilder, Tesselator,
};

pub(crate) struct MapMeshHandler {
    builders: MapBuilders,
    cache: Option<Arc<MapMeshes>>,
    pub(crate) map_dirt_id: u32,
    last_config: usize,
}

struct MapBuilders {
    buildsprites: FastMap<BuildingKind, SpriteBatchBuilder>,
    buildmeshes: FastMap<BuildingKind, InstancedMeshBuilder>,
    houses_mesh: MeshBuilder,
    zonemeshes: FastMap<BuildingKind, (MeshBuilder, InstancedMeshBuilder)>,
    arrow_builder: SpriteBatchBuilder,
    crosswalk_builder: MeshBuilder,
    tess_map: Tesselator,
}

pub(crate) struct MapMeshes {
    map: Option<Mesh>,
    crosswalks: Option<Mesh>,
    bsprites: MultiSpriteBatch,
    bmeshes: Vec<InstancedMesh>,
    houses_mesh: Option<Mesh>,
    zone_meshes: Vec<(Option<Mesh>, Option<InstancedMesh>)>,
    arrows: Option<SpriteBatch>,
    pub enable_arrows: AtomicBool,
}

impl MapMeshHandler {
    pub(crate) fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let arrow_builder = SpriteBatchBuilder::from_path(gfx, "assets/sprites/arrow_one_way.png");

        let mut buildsprites = FastMap::default();
        let mut buildmeshes = FastMap::default();
        let mut zonemeshes = FastMap::default();

        for descr in goria.read::<GoodsCompanyRegistry>().descriptions.values() {
            let asset = &descr.asset_location;
            if !asset.ends_with(".png") && !asset.ends_with(".jpg") {
                continue;
            }
            if descr.zone.is_some() {
                continue;
            }
            buildsprites.insert(
                BuildingKind::GoodsCompany(descr.id),
                SpriteBatchBuilder::new(gfx.texture(asset, "goods_company_tex")),
            );
        }

        for (asset, bkind) in goria
            .read::<GoodsCompanyRegistry>()
            .descriptions
            .values()
            .map(|descr| {
                (
                    descr.asset_location.as_ref(),
                    BuildingKind::GoodsCompany(descr.id),
                )
            })
            .chain([
                ("rail_fret_station.glb", BuildingKind::RailFretStation),
                ("trainstation.glb", BuildingKind::TrainStation),
                ("external_trading.glb", BuildingKind::ExternalTrading),
            ])
        {
            if !asset.ends_with(".glb") {
                continue;
            }
            let m = match load_mesh(gfx, asset) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to load mesh {}: {:?}", asset, e);
                    continue;
                }
            };

            buildmeshes.insert(bkind, InstancedMeshBuilder::new(m));
        }

        for descr in goria.read::<GoodsCompanyRegistry>().descriptions.values() {
            let Some(ref z) = descr.zone else { continue };
            let floor = &z.floor;
            let filler = &z.filler;

            let floor_mesh = MeshBuilder::new(gfx.texture(floor, "zone_floor_tex"));

            let m = match load_mesh(gfx, filler) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to load mesh for zone {}: {:?}", filler, e);
                    continue;
                }
            };

            let filler_mesh = InstancedMeshBuilder::new(m);

            zonemeshes.insert(
                BuildingKind::GoodsCompany(descr.id),
                (floor_mesh, filler_mesh),
            );
        }

        let builders = MapBuilders {
            arrow_builder,
            buildsprites,
            crosswalk_builder: MeshBuilder::new(
                gfx.texture("assets/sprites/crosswalk.png", "crosswalk"),
            ),
            tess_map: Tesselator::new(gfx, None, 15.0),
            houses_mesh: MeshBuilder::new(gfx.palette()),
            buildmeshes,
            zonemeshes,
        };

        Self {
            builders,
            cache: None,
            map_dirt_id: 0,
            last_config: common::config_id(),
        }
    }

    pub(crate) fn latest_mesh(
        &mut self,
        map: &Map,
        gfx: &mut GfxContext,
    ) -> &Option<Arc<MapMeshes>> {
        if map.dirt_id.0 != self.map_dirt_id || self.last_config != common::config_id() {
            self.builders.map_mesh(map);
            self.builders.arrows(map);
            self.builders.crosswalks(map);
            self.builders.bspritesmesh(map);
            self.builders.houses_mesh(map);
            self.builders.zone_mesh(map);

            self.last_config = common::config_id();
            self.map_dirt_id = map.dirt_id.0;

            let m = &mut self.builders.tess_map.meshbuilder;

            let meshes = MapMeshes {
                map: m.build(gfx),
                crosswalks: self.builders.crosswalk_builder.build(gfx),
                bsprites: self
                    .builders
                    .buildsprites
                    .values_mut()
                    .flat_map(|x| x.build(gfx))
                    .collect(),
                bmeshes: self
                    .builders
                    .buildmeshes
                    .values_mut()
                    .flat_map(|x| x.build(gfx))
                    .collect(),
                houses_mesh: self.builders.houses_mesh.build(gfx),
                zone_meshes: self
                    .builders
                    .zonemeshes
                    .values_mut()
                    .map(|(a, b)| (a.build(gfx), b.build(gfx)))
                    .collect(),
                arrows: self.builders.arrow_builder.build(gfx),
                enable_arrows: Default::default(),
            };

            self.cache = Some(Arc::new(meshes));
        }
        &self.cache
    }
}

impl MapBuilders {
    fn arrows(&mut self, map: &Map) {
        self.arrow_builder.clear();
        let lanes = map.lanes();
        let roads = map.roads();
        for road in roads.values() {
            let fade = (road.length()
                - 5.0
                - road.interface_from(road.src)
                - road.interface_from(road.dst))
            .mul(0.2)
            .clamp(0.0, 1.0);

            let r_lanes = road.lanes_iter().filter(|(_, kind)| kind.needs_arrows());
            let n_arrows = ((road.length() / 50.0) as i32).max(1);

            for (id, _) in r_lanes {
                let lane = &lanes[id];
                let l = lane.points.length();
                for i in 0..n_arrows {
                    let (mid, dir) = lane
                        .points
                        .point_dir_along(l * (1.0 + i as f32) / (1.0 + n_arrows as f32));

                    self.arrow_builder.push(
                        mid.up(0.03),
                        dir,
                        LinearColor::gray(0.3 + fade * 0.1),
                        (4.0, 4.0),
                    );
                }
            }
        }
    }

    fn crosswalks(&mut self, map: &Map) {
        let builder = &mut self.crosswalk_builder;
        builder.clear();

        let walking_w: f32 = LaneKind::Walking.width();

        let lanes = map.lanes();
        let intersections = map.intersections();
        for (inter_id, inter) in intersections {
            for turn in inter.turns() {
                let id = turn.id;

                if matches!(turn.kind, TurnKind::Crosswalk) {
                    let from = lanes[id.src].get_inter_node_pos(inter_id).up(0.01);
                    let to = lanes[id.dst].get_inter_node_pos(inter_id).up(0.01);

                    let l = (to - from).magn();

                    if l < walking_w {
                        continue;
                    }

                    let dir = (to - from) / l;
                    let perp = dir.perp_up() * CROSSWALK_WIDTH * 0.5;
                    let pos = from + dir * walking_w * 0.5;
                    let height = l - walking_w;

                    builder.extend_with(|vertices, add_index| {
                        let mk_v = |position: Vec3, uv: Vec2| MeshVertex {
                            position: position.into(),
                            uv: uv.into(),
                            normal: Vec3::Z,
                            color: [1.0; 4],
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
    }

    fn bspritesmesh(&mut self, map: &Map) {
        for v in self.buildsprites.values_mut() {
            v.clear();
        }

        for v in self.buildmeshes.values_mut() {
            v.instances.clear();
        }

        let buildings = &map.buildings();

        for building in buildings.values() {
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

    fn zone_mesh(&mut self, map: &Map) {
        self.zonemeshes.values_mut().for_each(|x| {
            x.0.clear();
            x.1.instances.clear();
        });

        for building in map.buildings().values() {
            let Some(zone) = &building.zone else { continue };
            let Some((zone_mesh, filler)) = self.zonemeshes.get_mut(&building.kind) else { continue };
            let zone = &zone.poly;

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

            let principal_axis = building.obb.axis()[0].normalize();

            let Some((mut min, mut max)) = minmax(zone.iter().map(|x| x.rotated_by(principal_axis.flipy()))) else { continue };
            min = min.rotated_by(principal_axis);
            max = max.rotated_by(principal_axis);

            let secondary_axis = -principal_axis.perpendicular();

            let principal_dist = (max - min).dot(principal_axis).abs();
            let secondary_dist = (max - min).dot(secondary_axis).abs();

            for principal_offset in (0..=(principal_dist as i32)).step_by(4) {
                for secondary_offset in (0..=(secondary_dist as i32)).step_by(4) {
                    let pos = min
                        + principal_axis * principal_offset as f32
                        + secondary_axis * secondary_offset as f32;
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

            zone_mesh.extend_with(|vertices, add_index| {
                for p in &zone.0 {
                    vertices.push(MeshVertex {
                        position: p.z(building.height + 0.05).into(),
                        normal: Vec3::Z,
                        uv: ((*p + avg) * 0.05).into(),
                        color: [1.0; 4],
                    });
                }

                earcut(&zone.0, |a, b, c| {
                    add_index(a as u32);
                    add_index(b as u32);
                    add_index(c as u32);
                });
            })
        }
    }

    fn houses_mesh(&mut self, map: &Map) {
        self.houses_mesh.clear();

        let buildings = &map.buildings();

        let mut projected = Polygon(Vec::with_capacity(10));

        for building in buildings.values() {
            for (face, col) in &building.mesh.faces {
                self.houses_mesh.extend_with(|vertices, add_index| {
                    let o = face[1];
                    let u = unwrap_ret!((face[0] - o).try_normalize());
                    let v = unwrap_ret!((face[2] - o).try_normalize());

                    let mut nor = u.cross(v);

                    let mut reverse = false;

                    if nor.z < 0.0 {
                        reverse = true;
                        nor = -nor;
                    }

                    projected.clear();
                    for &p in face {
                        let off = p - o;
                        projected.0.push(vec2(off.dot(u), off.dot(v)));

                        vertices.push(MeshVertex {
                            position: p.into(),
                            normal: nor,
                            uv: [0.0; 2],
                            color: col.into(),
                        })
                    }

                    projected.simplify();

                    earcut(&projected.0, |mut a, b, mut c| {
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
    }

    fn draw_rail(tess: &mut Tesselator, cut: &PolyLine3, off: f32, limits: bool) {
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
        for (v, dir) in cut.equipoints_dir(1.0, !limits) {
            let up = vec3(v.x, v.y, v.z + 0.04);
            tess.draw_polyline_full(
                [up, up + dir * 0.1].into_iter(),
                dir.xy(),
                dir.xy(),
                2.0,
                off,
            );
        }
    }

    fn map_mesh(&mut self, map: &Map) {
        let tess = &mut self.tess_map;
        tess.meshbuilder.clear();

        let low_col: LinearColor = common::config().road_low_col.into();
        let mid_col: LinearColor = common::config().road_mid_col.into();
        let hig_col: LinearColor = common::config().road_hig_col.into();
        let line_col: LinearColor = common::config().road_line_col.into();

        let inters = map.intersections();
        let lanes = map.lanes();
        let roads = map.roads();
        let lots = map.lots();
        let terrain = &map.terrain;

        for road in roads.values() {
            let cut = road.interfaced_points();

            road_pylons(&mut tess.meshbuilder, terrain, road);

            tess.normal.z = -1.0;
            tess.draw_polyline_full(
                cut.iter().map(|x| x.up(-0.3)),
                cut.first_dir().unwrap_or_default().xy(),
                cut.last_dir().unwrap_or_default().xy(),
                road.width,
                0.0,
            );
            tess.normal.z = 1.0;

            let draw_off = |tess: &mut Tesselator, col: LinearColor, w, off| {
                tess.set_color(col);
                tess.draw_polyline_full(
                    cut.as_slice().iter().copied(),
                    unwrap_ret!(cut.first_dir()).xy(),
                    unwrap_ret!(cut.last_dir()).xy(),
                    w,
                    off,
                );
            };

            let mut start = true;
            for l in road.lanes_iter().flat_map(|(l, _)| lanes.get(l)) {
                if l.kind.is_rail() {
                    let off = l.dist_from_bottom - road.width * 0.5 + LaneKind::Rail.width() * 0.5;
                    draw_off(tess, mid_col, LaneKind::Rail.width(), off);
                    Self::draw_rail(tess, cut, off, true);
                    start = true;
                    continue;
                }
                if start {
                    draw_off(tess, line_col, 0.25, l.dist_from_bottom - road.width * 0.5);
                    start = false;
                }
                draw_off(
                    tess,
                    match l.kind {
                        LaneKind::Walking => hig_col,
                        LaneKind::Parking => low_col,
                        _ => mid_col,
                    },
                    l.kind.width() - 0.25,
                    l.dist_from_bottom - road.width * 0.5 + l.kind.width() * 0.5,
                );
                draw_off(
                    tess,
                    line_col,
                    0.25,
                    l.dist_from_bottom - road.width * 0.5 + l.kind.width(),
                );
            }
        }

        // Intersections
        let mut p = Vec::with_capacity(8);
        let mut ppoly = unsafe { PolyLine3::new_unchecked(vec![]) };
        for inter in inters.values() {
            if inter.roads.is_empty() {
                tess.set_color(line_col);
                tess.draw_circle(inter.pos, 5.5);

                tess.set_color(mid_col);
                tess.draw_circle(inter.pos, 5.0);
                continue;
            }

            inter_pylon(&mut tess.meshbuilder, terrain, inter, roads);
            intersection_mesh(&mut tess.meshbuilder, inter, roads);

            // Walking corners
            for turn in inter
                .turns()
                .filter(|turn| matches!(turn.kind, TurnKind::WalkingCorner))
            {
                tess.set_color(line_col);
                let id = turn.id;

                let w = lanes[id.src].kind.width();

                let first_dir = -lanes[id.src].orientation_from(id.parent);
                let last_dir = lanes[id.dst].orientation_from(id.parent);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_full(p.iter().copied(), first_dir, last_dir, 0.25, w * 0.5);
                tess.draw_polyline_full(p.iter().copied(), first_dir, last_dir, 0.25, -w * 0.5);

                tess.set_color(hig_col);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, w - 0.25);
            }

            // Rail turns
            for turn in inter
                .turns()
                .filter(|turn| matches!(turn.kind, TurnKind::Rail))
            {
                ppoly.clear_extend(turn.points.as_slice());
                Self::draw_rail(tess, &ppoly, 0.0, false);
            }
        }

        // Lots
        for lot in lots.values() {
            let col = match lot.kind {
                LotKind::Unassigned => common::config().lot_unassigned_col,
                LotKind::Residential => common::config().lot_residential_col,
            };
            tess.set_color(col);
            tess.draw_filled_polygon(&lot.shape.corners, lot.height + 0.3);
        }
    }
}

impl Drawable for MapMeshes {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        self.map.draw(gfx, rp);
        self.bsprites.draw(gfx, rp);
        self.bmeshes.draw(gfx, rp);
        self.houses_mesh.draw(gfx, rp);
        self.zone_meshes.draw(gfx, rp);
        if self.enable_arrows.load(Ordering::SeqCst) {
            self.arrows.draw(gfx, rp);
        }
        self.crosswalks.draw(gfx, rp);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu_engine::wgpu::BindGroup,
    ) {
        self.map.draw_depth(gfx, rp, shadow_map, proj);
        self.bsprites.draw_depth(gfx, rp, shadow_map, proj);
        self.bmeshes.draw_depth(gfx, rp, shadow_map, proj);
        self.houses_mesh.draw_depth(gfx, rp, shadow_map, proj);
        self.zone_meshes.draw_depth(gfx, rp, shadow_map, proj);
        if self.enable_arrows.load(Ordering::SeqCst) {
            self.arrows.draw_depth(gfx, rp, shadow_map, proj);
        }
        self.crosswalks.draw_depth(gfx, rp, shadow_map, proj);
    }
}

fn add_polyon(
    mut meshb: &mut MeshBuilder,
    w: f32,
    PylonPosition {
        terrain_height,
        pos,
        dir,
    }: PylonPosition,
) {
    let color = LinearColor::from(common::config().road_pylon_col);
    let color: [f32; 4] = color.into();

    let up = pos.up(-0.2);
    let down = pos.xy().z(terrain_height);
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

    let mr = &mut meshb;
    let mut quad = move |a, b, c, d, nor| {
        mr.extend_with(move |vertices, add_idx| {
            let mut pvert = move |p: Vec3, normal: Vec3| {
                vertices.push(MeshVertex {
                    position: p.into(),
                    normal,
                    uv: [0.0; 2],
                    color,
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

fn road_pylons(meshb: &mut MeshBuilder, terrain: &Terrain, road: &Road) {
    for pylon in Road::pylons_positions(road.interfaced_points(), terrain) {
        add_polyon(meshb, road.width * 0.5, pylon);
    }
}

fn inter_pylon(meshb: &mut MeshBuilder, terrain: &Terrain, inter: &Intersection, roads: &Roads) {
    let h = unwrap_ret!(terrain.height(inter.pos.xy()));
    if (h - inter.pos.z).abs() <= 2.0 {
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
        avgp = inter.pos;
    }

    add_polyon(
        meshb,
        maxw,
        PylonPosition {
            terrain_height: h,
            pos: avgp,
            dir: Vec3::X,
        },
    );
}

fn intersection_mesh(meshb: &mut MeshBuilder, inter: &Intersection, roads: &Roads) {
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

        let src_orient = unwrap_cont!(firstdir).xy();

        let left = firstp.xy() - src_orient.perpendicular() * getw(road);

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

        let ang = (-src_orient).angle(dst_orient);

        const TURN_ANG_ADD: f32 = 0.29;
        const TURN_ANG_MUL: f32 = 0.36;
        const TURN_MUL: f32 = 0.46;

        let dist = (next_right - left).mag() * (TURN_ANG_ADD + ang.abs() * TURN_ANG_MUL) * TURN_MUL;

        let spline = Spline {
            from: left,
            to: next_right,
            from_derivative: -src_orient * dist,
            to_derivative: dst_orient * dist,
        };

        polygon.extend(spline.smart_points(1.0, 0.0, 1.0));
    }

    polygon.simplify();

    let col = LinearColor::from(common::config().road_mid_col).into();
    meshb.extend_with(|vertices, add_idx| {
        vertices.extend(polygon.iter().map(|pos| MeshVertex {
            position: pos.z(inter.pos.z - 0.001).into(),
            normal: Vec3::Z,
            uv: [0.0; 2],
            color: col,
        }));
        earcut(&polygon.0, |a, b, c| {
            add_idx(a as u32);
            add_idx(b as u32);
            add_idx(c as u32);
            add_idx(c as u32);
            add_idx(b as u32);
            add_idx(a as u32);
        });
    });
}
