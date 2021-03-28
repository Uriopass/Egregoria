use common::{
    FastMap, Z_ARROW, Z_CROSSWALK, Z_HOUSE, Z_INTER_BG, Z_LANE, Z_LANE_BG, Z_LOT, Z_SIDEWALK,
};
use egregoria::souls::goods_company::GoodsCompanyRegistry;
use egregoria::utils::Restrict;
use egregoria::Egregoria;
use geom::{vec2, LinearColor};
use map_model::{
    BuildingKind, Buildings, Intersections, LaneKind, Lanes, LotKind, Lots, Map, Roads, TurnKind,
    CROSSWALK_WIDTH,
};
use std::ops::Mul;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use wgpu_engine::wgpu::{RenderPass, RenderPipeline};
use wgpu_engine::{
    compile_shader, CompiledShader, Drawable, GfxContext, Mesh, MeshBuilder, MultiSpriteBatch,
    ShadedBatch, ShadedBatchBuilder, ShadedInstanceRaw, Shaders, SpriteBatch, SpriteBatchBuilder,
    Tesselator,
};

#[derive(Copy, Clone)]
struct Crosswalk;

impl Shaders for Crosswalk {
    fn vert_shader(device: &wgpu_engine::wgpu::Device) -> CompiledShader {
        compile_shader(device, "assets/shaders/crosswalk.vert", None)
    }

    fn frag_shader(device: &wgpu_engine::wgpu::Device) -> CompiledShader {
        compile_shader(device, "assets/shaders/crosswalk.frag", None)
    }
}

pub struct MapMeshAsyncBuilder {
    rebuild_snd: Sender<MapCpy>,
    builders: Arc<Mutex<MapBuilders>>,
    cache: Option<Rc<MapMeshes>>,
    map_dirt_id: u32,
    builder_dirt_id: u32,
    last_config: usize,
}

struct BuilderThread {
    rebuild_rcv: Receiver<MapCpy>,
    builders: Arc<Mutex<MapBuilders>>,
}

struct MapBuilders {
    buildings_builder: FastMap<BuildingKind, SpriteBatchBuilder>,
    arrow_builder: SpriteBatchBuilder,
    crosswalk_builder: ShadedBatchBuilder<Crosswalk>,
    tess: Tesselator,
    dirt_id: u32,
}

pub struct MapMeshes {
    map: Option<Mesh>,
    crosswalks: Option<ShadedBatch<Crosswalk>>,
    buildings: MultiSpriteBatch,
    arrows: Option<SpriteBatch>,
}

struct MapCpy {
    lanes: Lanes,
    roads: Roads,
    intersections: Intersections,
    buildings: Buildings,
    lots: Lots,
    dirt_id: u32,
}

impl MapMeshAsyncBuilder {
    pub fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let arrow_builder = SpriteBatchBuilder::from_path(gfx, "assets/arrow_one_way.png");

        gfx.register_pipeline::<ShadedBatch<Crosswalk>>();

        let mut buildings_builder = FastMap::default();

        for descr in goria.read::<GoodsCompanyRegistry>().descriptions.values() {
            buildings_builder.insert(
                descr.bkind,
                SpriteBatchBuilder::new(
                    gfx.texture(descr.asset_location, Some(descr.asset_location)),
                ),
            );
        }

        let (rebuild_snd, rebuild_rcv) = std::sync::mpsc::channel();

        let builders = Arc::new(Mutex::new(MapBuilders {
            arrow_builder,
            buildings_builder,
            crosswalk_builder: ShadedBatchBuilder::new(),
            tess: Tesselator::new(None, 15.0),
            dirt_id: 0,
        }));

        let t = BuilderThread {
            rebuild_rcv,
            builders: builders.clone(),
        };

        std::thread::spawn(move || loop {
            let mut map = unwrap_orr!(t.rebuild_rcv.recv(), return);
            while let Ok(map_rcv) = t.rebuild_rcv.try_recv() {
                map = map_rcv;
            }
            let mut builders = t.builders.lock().unwrap();

            builders.map_mesh(&map);
            builders.arrows(&map);
            builders.crosswalks(&map);
            builders.buildings_sprites(&map);
            builders.dirt_id = map.dirt_id;

            drop(builders);
        });

        Self {
            rebuild_snd,
            builders,
            cache: None,
            map_dirt_id: 0,
            builder_dirt_id: 0,
            last_config: common::config_id(),
        }
    }
    pub fn latest_mesh(&mut self, map: &Map, gfx: &GfxContext) -> &Option<Rc<MapMeshes>> {
        if map.dirt_id != self.map_dirt_id || self.last_config != common::config_id() {
            let _ = self.rebuild_snd.send(MapCpy {
                lanes: map.lanes().clone(),
                roads: map.roads().clone(),
                intersections: map.intersections().clone(),
                buildings: map.buildings().clone(),
                lots: map.lots().clone(),
                dirt_id: map.dirt_id,
            });

            self.last_config = common::config_id();
            self.map_dirt_id = map.dirt_id;
        }

        if let Ok(mut builders) = self.builders.try_lock() {
            if builders.dirt_id != self.builder_dirt_id {
                self.builder_dirt_id = builders.dirt_id;

                let m = std::mem::take(&mut builders.tess.meshbuilder);
                let meshes = MapMeshes {
                    map: m.build(gfx),
                    crosswalks: builders.crosswalk_builder.build(gfx),
                    buildings: builders
                        .buildings_builder
                        .values()
                        .into_iter()
                        .flat_map(|x| x.build(gfx))
                        .collect(),
                    arrows: builders.arrow_builder.build(gfx),
                };

                self.cache = Some(Rc::new(meshes));
            }
        }
        &self.cache
    }
}

impl Drawable for MapMeshes {
    fn create_pipeline(_: &GfxContext) -> RenderPipeline
    where
        Self: Sized,
    {
        panic!("create the pipelines of the components :-)")
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        if let Some(ref map) = self.map {
            map.draw(gfx, rp);
        }
        self.buildings.draw(gfx, rp);
        if let Some(ref arrows) = self.arrows {
            arrows.draw(gfx, rp);
        }
        if let Some(ref crosswalks) = self.crosswalks {
            crosswalks.draw(gfx, rp);
        }
    }
}

impl MapBuilders {
    fn arrows(&mut self, map: &MapCpy) {
        self.arrow_builder.clear();
        let lanes = &map.lanes;
        let roads = &map.roads;
        for road in roads.values() {
            let fade = (road.length()
                - 5.0
                - road.interface_from(road.src)
                - road.interface_from(road.dst))
            .mul(0.2)
            .restrict(0.0, 1.0);

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
                        (4.0, 4.0),
                    );
                }
            }
        }
    }

    fn crosswalks(&mut self, map: &MapCpy) {
        let builder = &mut self.crosswalk_builder;
        builder.instances.clear();

        let lanes = &map.lanes;
        let intersections = &map.intersections;
        for (inter_id, inter) in intersections {
            for turn in inter.turns() {
                let id = turn.id;

                if matches!(turn.kind, TurnKind::Crosswalk) {
                    let from = lanes[id.src].get_inter_node_pos(inter_id);
                    let to = lanes[id.dst].get_inter_node_pos(inter_id);

                    let l = (to - from).magnitude();

                    if l < 4.0 {
                        continue;
                    }

                    let dir = (to - from) / l;
                    let pos = from + dir * 2.25 + dir.perpendicular() * CROSSWALK_WIDTH * 0.5;
                    let height = l - 4.5;

                    builder.instances.push(ShadedInstanceRaw::new(
                        pos,
                        Z_CROSSWALK,
                        dir,
                        vec2(height, CROSSWALK_WIDTH),
                        LinearColor::WHITE,
                    ));
                }
            }
        }
    }

    fn buildings_sprites(&mut self, map: &MapCpy) {
        for v in self.buildings_builder.values_mut() {
            v.clear();
        }

        let buildings = &map.buildings;

        for building in buildings.values() {
            if let Some(x) = self.buildings_builder.get_mut(&building.kind) {
                let axis = building.obb.axis();
                let c = building.obb.center();
                let w = axis[0].magnitude();
                let d = axis[0] / w;
                let h = axis[1].magnitude();
                x.push(c, d, Z_HOUSE - f32::EPSILON, LinearColor::WHITE, (w, h));
            }
        }
    }

    fn map_mesh(&mut self, map: &MapCpy) {
        let tess = &mut self.tess;
        tess.meshbuilder = MeshBuilder::new();

        let low_col: LinearColor = common::config().road_low_col.into();
        let mid_col: LinearColor = common::config().road_mid_col.into();
        let hig_col: LinearColor = common::config().road_hig_col.into();
        let line_col: LinearColor = common::config().road_line_col.into();

        let inters = &map.intersections;
        let lanes = &map.lanes;
        let lots = &map.lots;

        for l in lanes.values() {
            tess.set_color(line_col);

            let or_src = l.orientation_from(l.src);
            let or_dst = -l.orientation_from(l.dst);

            tess.draw_polyline_with_dir(
                l.points.as_slice(),
                or_src,
                or_dst,
                Z_LANE_BG,
                l.kind.width() + 0.5,
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
                l.kind.width() - 0.5,
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

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, Z_LANE_BG, w + 0.5);

                tess.set_color(hig_col);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                let z = Z_SIDEWALK;

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, z, w - 0.5);
            }
        }

        // Buildings mesh
        for building in map.buildings.values() {
            for (p, col) in &building.mesh.faces {
                tess.set_color(*col);
                tess.draw_filled_polygon(p.as_slice(), Z_HOUSE);
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
