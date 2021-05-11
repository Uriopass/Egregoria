use common::{
    FastMap, Z_ARROW, Z_BSPRITE, Z_CROSSWALK, Z_INTER_BG, Z_LANE, Z_LANE_BG, Z_LOT, Z_SIDEWALK,
};
use egregoria::souls::goods_company::GoodsCompanyRegistry;
use egregoria::Egregoria;
use geom::{vec2, LinearColor, Polygon, Vec2, Vec3};
use map_model::{BuildingKind, LaneKind, LotKind, Map, TurnKind, CROSSWALK_WIDTH};
use std::ops::Mul;
use std::rc::Rc;
use wgpu_engine::earcut::earcut;
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::wgpu::RenderPass;
use wgpu_engine::{
    Drawable, GfxContext, InstancedMesh, InstancedMeshBuilder, Mesh, MeshBuilder, MeshInstance,
    MeshVertex, MultiSpriteBatch, SpriteBatch, SpriteBatchBuilder, Tesselator,
};

pub struct MapMeshHandler {
    builders: MapBuilders,
    cache: Option<Rc<MapMeshes>>,
    map_dirt_id: u32,
    last_config: usize,
}

struct MapBuilders {
    buildsprites: FastMap<BuildingKind, SpriteBatchBuilder>,
    buildmeshes: FastMap<BuildingKind, InstancedMeshBuilder>,
    houses_mesh: MeshBuilder,
    arrow_builder: SpriteBatchBuilder,
    crosswalk_builder: MeshBuilder,
    tess: Tesselator,
}

pub struct MapMeshes {
    map: Option<Mesh>,
    crosswalks: Option<Mesh>,
    bsprites: MultiSpriteBatch,
    bmeshes: Vec<InstancedMesh>,
    houses_mesh: Option<Mesh>,
    arrows: Option<SpriteBatch>,
}

impl MapMeshHandler {
    pub fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let arrow_builder = SpriteBatchBuilder::from_path(gfx, "assets/arrow_one_way.png");

        let mut buildsprites = FastMap::default();
        let mut buildmeshes = FastMap::default();

        for descr in goria.read::<GoodsCompanyRegistry>().descriptions.values() {
            let asset = descr.asset_location;
            if !asset.ends_with(".png") {
                continue;
            }
            buildsprites.insert(
                descr.bkind,
                SpriteBatchBuilder::new(gfx.texture(asset, asset)),
            );
        }

        for descr in goria.read::<GoodsCompanyRegistry>().descriptions.values() {
            let asset = descr.asset_location;
            if !asset.ends_with(".glb") {
                continue;
            }
            buildmeshes.insert(
                descr.bkind,
                InstancedMeshBuilder::new(unwrap_contlog!(
                    load_mesh(asset, gfx),
                    "couldn't load obj: {}",
                    asset
                )),
            );
        }

        let builders = MapBuilders {
            arrow_builder,
            buildsprites,
            crosswalk_builder: MeshBuilder::new(),
            tess: Tesselator::new(None, 15.0),
            houses_mesh: MeshBuilder::new(),
            buildmeshes,
        };

        Self {
            builders,
            cache: None,
            map_dirt_id: 0,
            last_config: common::config_id(),
        }
    }

    pub fn latest_mesh(&mut self, map: &Map, gfx: &mut GfxContext) -> &Option<Rc<MapMeshes>> {
        if map.dirt_id.0 != self.map_dirt_id || self.last_config != common::config_id() {
            self.builders.map_mesh(&map);
            self.builders.arrows(&map);
            self.builders.crosswalks(&map);
            self.builders.bspritesmesh(&map);
            self.builders.houses_mesh(&map);

            self.last_config = common::config_id();
            self.map_dirt_id = map.dirt_id.0;

            let m = &mut self.builders.tess.meshbuilder;

            let cw = gfx.texture("assets/crosswalk.png", "crosswalk");

            let meshes = MapMeshes {
                map: m.build(gfx, gfx.palette()),
                crosswalks: self.builders.crosswalk_builder.build(gfx, cw),
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
                houses_mesh: self.builders.houses_mesh.build(gfx, gfx.palette()),
                arrows: self.builders.arrow_builder.build(gfx),
            };

            self.cache = Some(Rc::new(meshes));
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

            let r_lanes = road.lanes_iter().filter(|(_, kind)| kind.vehicles());
            let n_arrows = ((road.length() / 50.0) as i32).max(1);

            for (id, _) in r_lanes {
                let lane = &lanes[id];
                let l = lane.length();
                for i in 0..n_arrows {
                    let (mid, dir) = lane
                        .points
                        .point_dir_along(l * (1.0 + i as f32) / (1.0 + n_arrows as f32));

                    self.arrow_builder.push(
                        mid,
                        dir,
                        Z_ARROW,
                        LinearColor::gray(0.3 + fade * 0.1),
                        (2.0, 2.0),
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
                    let from = lanes[id.src].get_inter_node_pos(inter_id);
                    let to = lanes[id.dst].get_inter_node_pos(inter_id);

                    let l = (to - from).magnitude();

                    if l < walking_w {
                        continue;
                    }

                    let dir = (to - from) / l;
                    let perp = dir.perpendicular() * CROSSWALK_WIDTH * 0.5;
                    let pos = (from + dir * walking_w * 0.5).z(Z_CROSSWALK);
                    let height = l - walking_w;
                    let dir = dir.z(0.0);
                    let perp = perp.z(0.0);

                    builder.extend_with(|vertices, add_index| {
                        let mk_v = |position: Vec3, uv: Vec2| MeshVertex {
                            position: position.into(),
                            uv: uv.into(),
                            normal: [0.0, 0.0, 1.0],
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
                let w = axis[0].magnitude();
                let d = axis[0] / w;
                let h = axis[1].magnitude();
                x.push(c, d, Z_BSPRITE, LinearColor::WHITE, (w, h));
            }

            if let Some(x) = self.buildmeshes.get_mut(&building.kind) {
                let pos = building.obb.center().z(Z_BSPRITE);
                let dir = building.obb.axis()[0].normalize().z(0.0);

                x.instances.push(MeshInstance {
                    pos,
                    dir,
                    tint: LinearColor::WHITE,
                });
            }
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
                            normal: nor.into(),
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

    fn map_mesh(&mut self, map: &Map) {
        let tess = &mut self.tess;
        tess.meshbuilder.clear();

        let low_col: LinearColor = common::config().road_low_col.into();
        let mid_col: LinearColor = common::config().road_mid_col.into();
        let hig_col: LinearColor = common::config().road_hig_col.into();
        let line_col: LinearColor = common::config().road_line_col.into();

        let inters = map.intersections();
        let lanes = map.lanes();
        let lots = map.lots();

        for l in lanes.values() {
            tess.set_color(line_col);

            let or_src = l.orientation_from(l.src);
            let or_dst = -l.orientation_from(l.dst);

            tess.draw_polyline_with_dir(
                l.points.as_slice(),
                or_src,
                or_dst,
                Z_LANE_BG,
                l.kind.width() + 0.25,
            );

            tess.set_color(match l.kind {
                LaneKind::Walking => hig_col,
                LaneKind::Parking => low_col,
                _ => mid_col,
            });
            let z = match l.kind {
                LaneKind::Walking => Z_SIDEWALK,
                _ => Z_LANE,
            };

            tess.draw_polyline_with_dir(
                l.points.as_slice(),
                or_src,
                or_dst,
                z,
                l.kind.width() - 0.25,
            );
        }

        // Intersections
        let mut p = Vec::with_capacity(8);
        for inter in inters.values() {
            if inter.roads.is_empty() {
                tess.set_color(line_col);
                tess.draw_circle(inter.pos, Z_LANE_BG, 5.5);

                tess.set_color(mid_col);
                tess.draw_circle(inter.pos, Z_LANE, 5.0);
                continue;
            }

            tess.set_color(mid_col);
            tess.draw_filled_polygon(inter.polygon.as_slice(), Z_INTER_BG);

            // Walking corners
            for turn in inter
                .turns()
                .iter()
                .filter(|turn| matches!(turn.kind, TurnKind::WalkingCorner))
            {
                tess.set_color(line_col);
                let id = turn.id;

                let w = lanes[id.src].kind.width();

                let first_dir = -lanes[id.src].orientation_from(id.parent);
                let last_dir = lanes[id.dst].orientation_from(id.parent);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, Z_LANE_BG, w + 0.25);

                tess.set_color(hig_col);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                let z = Z_SIDEWALK;

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, z, w - 0.25);
            }
        }

        // Lots
        for lot in lots.values() {
            let col = match lot.kind {
                LotKind::Unassigned => common::config().lot_unassigned_col,
                LotKind::Residential => common::config().lot_residential_col,
            };
            tess.set_color(col);
            tess.draw_filled_polygon(&lot.shape.corners, Z_LOT);
        }
    }
}

impl Drawable for MapMeshes {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        if let Some(ref map) = self.map {
            map.draw(gfx, rp);
        }
        self.bsprites.draw(gfx, rp);
        for v in &self.bmeshes {
            v.draw(gfx, rp);
        }
        if let Some(ref bmesh) = self.houses_mesh {
            bmesh.draw(gfx, rp);
        }
        if let Some(ref arrows) = self.arrows {
            arrows.draw(gfx, rp);
        }
        if let Some(ref crosswalks) = self.crosswalks {
            crosswalks.draw(gfx, rp);
        }
    }

    fn draw_depth<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        if let Some(ref map) = self.map {
            map.draw_depth(gfx, rp);
        }
        self.bsprites.draw_depth(gfx, rp);
        for v in &self.bmeshes {
            v.draw_depth(gfx, rp);
        }
        if let Some(ref bmesh) = self.houses_mesh {
            bmesh.draw_depth(gfx, rp);
        }
        if let Some(ref arrows) = self.arrows {
            arrows.draw_depth(gfx, rp);
        }
        if let Some(ref crosswalks) = self.crosswalks {
            crosswalks.draw_depth(gfx, rp);
        }
    }
}
