use common::{
    Z_ARROW, Z_CROSSWALK, Z_HOUSE, Z_INTER_BG, Z_LANE, Z_LANE_BG, Z_LOT, Z_SIDEWALK, Z_SIGNAL,
    Z_TREE, Z_TREE_SHADOW,
};
use egregoria::souls::goods_company::GoodsCompanyRegistry;
use egregoria::utils::Restrict;
use egregoria::Egregoria;
use flat_spatial::storage::Storage;
use geom::{lerp, vec2, Color, LinearColor, AABB};
use map_model::{
    BuildingKind, Lane, LaneKind, LotKind, Map, ProjectKind, TrafficBehavior, TurnKind,
    CROSSWALK_WIDTH,
};
use std::collections::HashMap;
use std::ops::Mul;
use wgpu_engine::{
    compile_shader, CompiledShader, FrameContext, GfxContext, Mesh, MultiSpriteBatch,
    MultiSpriteBatchBuilder, ShadedBatch, ShadedBatchBuilder, ShadedInstanceRaw, Shaders,
    SpriteBatch, SpriteBatchBuilder, Tesselator,
};

#[derive(Copy, Clone)]
struct Crosswalk;

impl Shaders for Crosswalk {
    fn vert_shader() -> CompiledShader {
        compile_shader("assets/shaders/crosswalk.vert", None)
    }

    fn frag_shader() -> CompiledShader {
        compile_shader("assets/shaders/crosswalk.frag", None)
    }
}

pub struct RoadRenderer {
    map_mesh: Option<Mesh>,
    buildings_builder: HashMap<BuildingKind, SpriteBatchBuilder>,
    buildings: Option<MultiSpriteBatch>,
    arrows: Option<SpriteBatch>,
    arrow_builder: SpriteBatchBuilder,
    tree_shadows: Option<SpriteBatch>,
    tree_shadows_builder: SpriteBatchBuilder,
    last_cam: AABB,
    trees: Option<MultiSpriteBatch>,
    tree_builder: MultiSpriteBatchBuilder,
    crosswalks: Option<ShadedBatch<Crosswalk>>,
    last_config: usize,
    map_dirt_id: u32,
    trees_dirt_id: u32,
}

impl RoadRenderer {
    pub fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        let arrow_builder = SpriteBatchBuilder::from_path(gfx, "assets/arrow_one_way.png");

        gfx.register_pipeline::<ShadedBatch<Crosswalk>>();

        let tree_builder = MultiSpriteBatchBuilder::from_paths(
            gfx,
            &[
                "assets/tree.png",
                "assets/tree2.png",
                "assets/tree3.png",
                "assets/tree4.png",
                "assets/tree5.png",
                "assets/tree6.png",
                "assets/tree7.png",
            ],
        );
        let tree_shadow_builder = SpriteBatchBuilder::from_path(gfx, "assets/tree_shadow.png");

        let mut buildings_builder = HashMap::new();

        for descr in goria.read::<GoodsCompanyRegistry>().descriptions.values() {
            buildings_builder.insert(
                descr.bkind,
                SpriteBatchBuilder::new(
                    gfx.texture(descr.asset_location, Some(descr.asset_location)),
                ),
            );
        }

        RoadRenderer {
            map_mesh: None,
            buildings_builder,
            buildings: None,
            arrows: None,
            arrow_builder,
            tree_shadows: None,
            tree_shadows_builder: tree_shadow_builder,
            last_cam: AABB::zero(),
            trees: None,
            tree_builder,
            crosswalks: None,
            last_config: common::config_id(),
            map_dirt_id: 0,
            trees_dirt_id: 0,
        }
    }

    fn map_mesh(map: &Map, mut tess: Tesselator, gfx: &GfxContext) -> Option<Mesh> {
        let low_col: LinearColor = common::config().road_low_col.into();
        let mid_col: LinearColor = common::config().road_mid_col.into();
        let hig_col: LinearColor = common::config().road_hig_col.into();
        let line_col: LinearColor = common::config().road_line_col.into();

        let inters = map.intersections();
        let lanes = map.lanes();

        for l in lanes.values() {
            tess.set_color(line_col);

            let or_src = l.orientation_from(l.src);
            let or_dst = -l.orientation_from(l.dst);

            tess.draw_polyline_with_dir(
                l.points.as_slice(),
                or_src,
                or_dst,
                Z_LANE_BG,
                l.width + 0.5,
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

            tess.draw_polyline_with_dir(l.points.as_slice(), or_src, or_dst, z, l.width - 0.5);
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

                let w = lanes[id.src].width;

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
        for building in map.buildings().values() {
            for (p, col) in &building.mesh.faces {
                tess.set_color(*col);
                tess.draw_filled_polygon(p.as_slice(), Z_HOUSE);
            }
        }

        // Lots
        for lot in map.lots().values() {
            let col = match lot.kind {
                LotKind::Unassigned => common::config().lot_unassigned_col,
                LotKind::Residential => common::config().lot_residential_col,
            };
            tess.set_color(col);
            tess.draw_filled_polygon(&lot.shape.corners, Z_LOT);
        }
        tess.meshbuilder.build(gfx)
    }

    fn buildings_sprites(&mut self, map: &Map, gfx: &GfxContext) -> MultiSpriteBatch {
        for v in self.buildings_builder.values_mut() {
            v.clear();
        }

        for building in map.buildings().values() {
            if let Some(x) = self.buildings_builder.get_mut(&building.kind) {
                let axis = building.obb.axis();
                let c = building.obb.center();
                let w = axis[0].magnitude();
                let d = axis[0] / w;
                let h = axis[1].magnitude();
                x.push(
                    c,
                    d,
                    Z_HOUSE - std::f32::EPSILON,
                    LinearColor::WHITE,
                    (w, h),
                );
            }
        }

        self.buildings_builder
            .values()
            .flat_map(|x| x.build(gfx))
            .collect()
    }

    fn render_lane_signals(n: &Lane, sr: &mut Tesselator, time: u32) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + dir_perp * -3.5 + dir * -1.0;

        // Stop sign
        if n.control.is_stop_sign() {
            sr.set_color(LinearColor::WHITE);
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.5, 8, std::f32::consts::FRAC_PI_8);

            sr.set_color(LinearColor::RED);
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.4, 8, std::f32::consts::FRAC_PI_8);
            return;
        }

        // Traffic light
        let size = 0.5; // light size

        sr.color = Color::gray(0.2).into();
        sr.draw_rect_cos_sin(r_center, Z_SIGNAL, size + 0.1, size * 3.0 + 0.1, dir);

        for i in -1..2 {
            sr.draw_circle(r_center + i as f32 * dir_perp * size, Z_SIGNAL, size * 0.5);
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

        sr.draw_circle(r_center + offset * dir_perp, Z_SIGNAL, size * 0.5);
    }

    fn signals_render(map: &Map, time: u32, sr: &mut Tesselator) {
        match sr.cull_rect {
            Some(rect) => {
                if rect.w().max(rect.h()) > 1500.0 {
                    return;
                }
                for n in map
                    .spatial_map()
                    .query(rect)
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

    fn arrows(&mut self, map: &Map, gfx: &GfxContext) -> Option<SpriteBatch> {
        self.arrow_builder.clear();
        let lanes = map.lanes();
        for road in map.roads().values() {
            let fade = (road.length - 5.0 - road.src_interface - road.dst_interface)
                .mul(0.2)
                .restrict(0.0, 1.0);

            let r_lanes = road.lanes_iter().filter(|(_, kind)| kind.vehicles());
            let n_arrows = ((road.length / 50.0) as i32).max(1);

            for (id, _) in r_lanes {
                let lane = &lanes[id];
                let l = lane.length;
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
        self.arrow_builder.build(gfx)
    }

    fn crosswalks(map: &Map, gfx: &GfxContext) -> Option<ShadedBatch<Crosswalk>> {
        let mut builder = ShadedBatchBuilder::<Crosswalk>::new();

        let lanes = map.lanes();
        for (inter_id, inter) in map.intersections() {
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
        builder.build(gfx)
    }

    pub fn trees(
        &mut self,
        map: &Map,
        screen: AABB,
        gfx: &GfxContext,
    ) -> (MultiSpriteBatch, Option<SpriteBatch>) {
        let st = map.trees.grid.storage();
        if map.trees.dirt_id == self.trees_dirt_id
            && self.tree_shadows.is_some()
            && st.cell_id(screen.ll) == st.cell_id(self.last_cam.ll)
            && st.cell_id(screen.ur) == st.cell_id(self.last_cam.ur)
        {
            if let Some(trees) = self.trees.as_ref() {
                return (trees.clone(), self.tree_shadows.clone());
            }
        }

        self.trees_dirt_id = map.trees.dirt_id;

        self.tree_builder.clear();
        self.tree_shadows_builder.clear();

        let k = screen.w().min(screen.h());
        if k > 4500.0 {
            return (
                self.tree_builder.build(gfx),
                self.tree_shadows_builder.build(gfx),
            );
        }

        let alpha_cutoff = lerp(1.0, 0.0, (k - 3000.0) / 1500.0);

        let tree_col = LinearColor::from(common::config().tree_col).a(alpha_cutoff);

        for (h, _) in map.trees.grid.query_raw(screen.ll, screen.ur) {
            let (pos, t) = map.trees.grid.get(h).unwrap();

            self.tree_shadows_builder.push(
                pos + vec2(1.0, -1.0),
                t.dir,
                Z_TREE_SHADOW,
                LinearColor::WHITE.a(alpha_cutoff),
                (t.size, t.size),
            );

            self.tree_builder
                .sb(
                    (common::rand::rand3(pos.x, pos.y, 10.0) * self.tree_builder.n_texs() as f32)
                        as usize,
                )
                .push(pos, t.dir, Z_TREE, t.col * tree_col, (t.size, t.size));
        }

        (
            self.tree_builder.build(gfx),
            self.tree_shadows_builder.build(gfx),
        )
    }

    pub fn render(&mut self, map: &Map, time: u32, tess: &mut Tesselator, ctx: &mut FrameContext) {
        let screen = tess
            .cull_rect
            .expect("no cull rectangle, might render far too many trees");
        if map.dirt_id != self.map_dirt_id || self.last_config != common::config_id() {
            self.map_mesh = Self::map_mesh(map, Tesselator::new(None, 15.0), ctx.gfx);
            self.arrows = self.arrows(map, ctx.gfx);
            self.crosswalks = Self::crosswalks(map, ctx.gfx);
            self.buildings = Some(self.buildings_sprites(map, ctx.gfx));

            self.last_config = common::config_id();
            self.map_dirt_id = map.dirt_id;
        }

        let (trees, tree_shadows) = self.trees(map, screen, ctx.gfx);
        self.trees = Some(trees);
        self.tree_shadows = tree_shadows;

        if let Some(x) = self.buildings.clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.map_mesh.clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.arrows.clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.crosswalks.clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.tree_shadows.clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.trees.clone() {
            ctx.draw(x);
        }

        Self::signals_render(map, time, tess);

        self.last_cam = screen;
    }
}
