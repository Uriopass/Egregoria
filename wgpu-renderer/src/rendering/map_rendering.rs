use crate::engine::{
    compile_shader, CompiledShader, FrameContext, GfxContext, InstanceRaw, Mesh, ShadedBatch,
    ShadedBatchBuilder, ShadedInstanceRaw, Shaders, SpriteBatch, SpriteBatchBuilder, Texture,
};
use crate::geometry::Tesselator;
use egregoria::physics::Transform;
use egregoria::rendering::{from_srgb, Color, LinearColor};
use egregoria::utils::Restrict;
use map_model::{Lane, LaneKind, Map, ProjectKind, TrafficBehavior, TurnKind, CROSSWALK_WIDTH};
use std::ops::Mul;

#[derive(Clone, Copy)]
struct Crosswalk;

impl Shaders for Crosswalk {
    fn vert_shader() -> CompiledShader {
        compile_shader("resources/shaders/crosswalk.vert", None)
    }

    fn frag_shader() -> CompiledShader {
        compile_shader("resources/shaders/crosswalk.frag", None)
    }
}

pub struct RoadRenderer {
    map_mesh: Option<Mesh>,
    arrows: Option<SpriteBatch>,
    arrow_builder: SpriteBatchBuilder,
    crosswalks: Option<ShadedBatch<Crosswalk>>,
}

const Z_INTER_BG: f32 = 0.20;
const Z_LANE_BG: f32 = 0.21;
const Z_LANE: f32 = 0.22;
const Z_SIDEWALK: f32 = 0.23;
const Z_ARROW: f32 = 0.24;
const Z_CROSSWALK: f32 = 0.25;
const Z_SIGNAL: f32 = 0.26;
const Z_HOUSE: f32 = 0.3;

const MID_GRAY_V: f32 = 0.5;

impl RoadRenderer {
    pub fn new(gfx: &mut GfxContext) -> Self {
        let arrow_builder = SpriteBatchBuilder::new(
            Texture::from_path(gfx, "resources/arrow_one_way.png", Some("arrow")).unwrap(), // Unwrap ok: File is there
        );

        gfx.register_pipeline::<ShadedBatch<Crosswalk>>();

        RoadRenderer {
            map_mesh: None,
            arrows: None,
            arrow_builder,
            crosswalks: None,
        }
    }

    fn map_mesh(&self, map: &Map, mut tess: Tesselator, gfx: &GfxContext) -> Option<Mesh> {
        let low_gray: LinearColor = Color::gray(0.3).into();
        let mid_gray: LinearColor = Color::gray(MID_GRAY_V).into();
        let high_gray: LinearColor = Color::gray(0.7).into();

        let inters = map.intersections();
        let lanes = map.lanes();

        tess.color = LinearColor::WHITE;
        for l in lanes.values() {
            tess.color = LinearColor::WHITE;

            let or_src = l.orientation_from(l.src);
            let or_dst = -l.orientation_from(l.dst);

            let w = l.width + 0.5;
            tess.draw_polyline_with_dir(l.points.as_slice(), or_src, or_dst, Z_LANE_BG, w);

            tess.color = match l.kind {
                LaneKind::Walking => high_gray,
                LaneKind::Parking => low_gray,
                _ => mid_gray,
            };
            let z = match l.kind {
                LaneKind::Walking => Z_SIDEWALK,
                _ => Z_LANE,
            };

            tess.draw_polyline_with_dir(l.points.as_slice(), or_src, or_dst, z, l.width - 0.5);
        }

        let mut p = Vec::with_capacity(8);
        for inter in inters.values() {
            if inter.roads.is_empty() {
                tess.color = LinearColor::WHITE;
                tess.draw_circle(inter.pos, Z_LANE_BG, 5.5);

                tess.color = mid_gray;
                tess.draw_circle(inter.pos, Z_LANE, 5.0);
                continue;
            }

            tess.color = mid_gray;
            tess.draw_filled_polygon(inter.polygon.as_slice(), Z_INTER_BG);

            for turn in inter
                .turns()
                .iter()
                .filter(|turn| matches!(turn.kind, TurnKind::WalkingCorner))
            {
                tess.color = LinearColor::WHITE;
                let id = turn.id;

                let w = lanes[id.src].width;

                let first_dir = -lanes[id.src].orientation_from(id.parent);
                let last_dir = lanes[id.dst].orientation_from(id.parent);

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, Z_LANE_BG, w + 0.5);

                tess.color = high_gray;

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                let z = Z_SIDEWALK;

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, z, w - 0.5);
            }
        }

        tess.color = LinearColor::gray(from_srgb(0.4));
        for house in map.houses().values() {
            tess.draw_filled_polygon(house.exterior.as_slice(), Z_HOUSE);
        }
        tess.meshbuilder.build(gfx)
    }

    fn render_lane_signals(n: &Lane, sr: &mut Tesselator, time: u64) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + dir_perp * 2.5 + dir * 2.5;

        if n.control.is_stop_sign() {
            sr.color = LinearColor::WHITE;
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.5, 8, std::f32::consts::FRAC_PI_8);

            sr.color = LinearColor::RED;
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.4, 8, std::f32::consts::FRAC_PI_8);
            return;
        }

        let size = 0.5; // light size

        sr.color = Color::gray(0.3).into();
        sr.draw_rect_cos_sin(r_center, Z_SIGNAL, size + 0.1, size * 3.0 + 0.1, dir);

        for i in -1..2 {
            sr.draw_circle(r_center + i as f32 * dir_perp * size, Z_SIGNAL, size * 0.5);
        }
        sr.color = match n.control.get_behavior(time) {
            TrafficBehavior::RED | TrafficBehavior::STOP => LinearColor::RED,
            TrafficBehavior::ORANGE => LinearColor::ORANGE,
            TrafficBehavior::GREEN => LinearColor::GREEN,
        };

        let offset = match n.control.get_behavior(time) {
            TrafficBehavior::RED => -size,
            TrafficBehavior::ORANGE => 0.0,
            TrafficBehavior::GREEN => size,
            _ => unreachable!(),
        };

        sr.draw_circle(r_center + offset * dir_perp, Z_SIGNAL, size * 0.5);
    }

    fn signals_render(map: &Map, time: u64, sr: &mut Tesselator) {
        match sr.cull_rect {
            Some(rect) => {
                if rect.w.max(rect.h) > 1500.0 {
                    return;
                }
                for n in map
                    .spatial_map()
                    .query_rect(rect)
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
        self.arrow_builder.instances.clear();
        let lanes = map.lanes();
        for road in map.roads().values() {
            let fade = (road.length - 5.0 - road.src_interface - road.dst_interface)
                .mul(0.2)
                .restrict(0.0, 1.0);

            let r_lanes = road.lanes_iter().filter(|(_, kind)| kind.vehicles());
            let n_arrows = ((road.length / 50.0) as i32).max(1);

            for (id, _) in r_lanes {
                let lane = &lanes[id];
                let l = lane.points.length();
                for i in 0..n_arrows {
                    let (mid, dir) = lane
                        .points
                        .point_dir_along(l * (1.0 + i as f32) / (1.0 + n_arrows as f32));

                    self.arrow_builder.instances.push(InstanceRaw::new(
                        Transform::new_cos_sin(mid, dir).to_matrix4(Z_ARROW),
                        [from_srgb(MID_GRAY_V) + fade * 0.6; 3],
                        4.0,
                    ));
                }
            }
        }
        self.arrow_builder.build(gfx)
    }

    fn crosswalks(&mut self, map: &Map, gfx: &GfxContext) -> Option<ShadedBatch<Crosswalk>> {
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

                    let t = Transform::new_cos_sin(from + dir * 2.25, dir);
                    let mut m = t.to_matrix4(Z_CROSSWALK);

                    m.x.x *= l - 4.5;
                    m.x.y *= l - 4.5;

                    m.y.x *= CROSSWALK_WIDTH;
                    m.y.y *= CROSSWALK_WIDTH;

                    builder
                        .instances
                        .push(ShadedInstanceRaw::new(m, [1.0, 1.0, 1.0, 1.0]));
                }
            }
        }
        builder.build(&gfx)
    }

    pub fn render(
        &mut self,
        map: &mut Map,
        time: u64,
        tess: &mut Tesselator,
        ctx: &mut FrameContext,
    ) {
        if map.dirty || self.map_mesh.is_none() {
            self.map_mesh = self.map_mesh(map, Tesselator::new(None, 15.0), &ctx.gfx);
            self.arrows = self.arrows(map, &ctx.gfx);
            self.crosswalks = self.crosswalks(map, &ctx.gfx);

            map.dirty = false;
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

        Self::signals_render(map, time, tess);
    }
}
