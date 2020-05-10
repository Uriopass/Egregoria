use crate::engine::{
    compile_shader, CompiledShader, FrameContext, GfxContext, InstanceRaw, Mesh, ShadedBatch,
    ShadedBatchBuilder, ShadedInstanceRaw, Shaders, SpriteBatch, SpriteBatchBuilder, Texture,
};
use crate::geometry::Tesselator;
use cgmath::{vec2, InnerSpace, Vector2};
use scale::geometry::Vec2Impl;
use scale::map_model::{LaneKind, Map, TrafficBehavior, TurnKind};
use scale::physics::Transform;
use scale::rendering::{from_srgb, LinearColor};
use scale::utils::Restrict;
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
    road_mesh: Option<Mesh>,
    arrows: Option<SpriteBatch>,
    arrow_builder: SpriteBatchBuilder,
    crosswalks: Option<ShadedBatch<Crosswalk>>,
}

const Z_LANE_BG: f32 = 0.21;
const Z_SIDEWALK: f32 = 0.22;
const Z_LANE: f32 = 0.23;
const Z_ARROW: f32 = 0.24;
const Z_CROSSWALK: f32 = 0.25;
const Z_SIGNAL: f32 = 0.26;

const MID_GRAY_V: f32 = 0.5;

impl RoadRenderer {
    pub fn new(gfx: &mut GfxContext) -> Self {
        let arrow_builder = SpriteBatchBuilder::new(
            Texture::from_path(gfx, "resources/arrow_one_way.png", Some("arrow")).unwrap(),
        );

        gfx.register_pipeline::<ShadedBatch<Crosswalk>>();

        RoadRenderer {
            road_mesh: None,
            arrows: None,
            arrow_builder,
            crosswalks: None,
        }
    }

    fn road_mesh(map: &Map, mut tess: Tesselator, gfx: &GfxContext) -> Option<Mesh> {
        let mid_gray: LinearColor = LinearColor::gray(MID_GRAY_V);
        let high_gray: LinearColor = LinearColor::gray(0.7);

        let inters = map.intersections();
        let lanes = map.lanes();

        tess.color = LinearColor::WHITE;

        let mut p = Vec::with_capacity(8);

        for n in lanes.values() {
            tess.color = LinearColor::WHITE;

            let first = n.points.first().unwrap();
            let last = n.points.last().unwrap();

            let w = n.width + 0.5;
            tess.draw_stroke(first, last, Z_LANE_BG, w);

            tess.color = match n.kind {
                LaneKind::Walking => high_gray,
                _ => mid_gray,
            };
            let z = match n.kind {
                LaneKind::Walking => Z_SIDEWALK,
                _ => Z_LANE,
            };

            tess.draw_stroke(first, last, z, n.width - 0.5);
        }

        for (inter_id, inter) in inters {
            if inter.roads.is_empty() {
                tess.color = LinearColor::WHITE;
                tess.draw_circle(inter.pos, Z_LANE_BG, 5.5);

                tess.color = mid_gray;
                tess.draw_circle(inter.pos, Z_LANE, 5.0);
            }
            for turn in inter.turns() {
                if matches!(turn.kind, TurnKind::Crosswalk) {
                    continue;
                }

                tess.color = LinearColor::WHITE;
                let id = turn.id;

                let w = lanes[id.src].width;

                let first_dir = lanes[id.src].get_orientation_vec();
                let last_dir = lanes[id.dst].get_orientation_vec();

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, Z_LANE_BG, w + 0.5);

                tess.color = match turn.kind {
                    TurnKind::WalkingCorner => high_gray,
                    _ => mid_gray,
                };

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                let z = match turn.kind {
                    TurnKind::WalkingCorner => Z_SIDEWALK,
                    _ => Z_LANE,
                };

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, z, w - 0.5);
            }
        }
        tess.meshbuilder.build(gfx)
    }

    fn signals_render(map: &Map, time: u64, sr: &mut Tesselator) {
        for n in map.lanes().values() {
            if n.control.is_always() {
                continue;
            }

            let dir = n.get_orientation_vec();

            let dir_nor = vec2(dir.y, -dir.x);

            let r_center = n.points.last().unwrap() + dir_nor * 2.5 + dir * 2.5;

            if n.control.is_stop_sign() {
                sr.color = LinearColor::WHITE;
                sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.5, 8, std::f32::consts::FRAC_PI_8);

                sr.color = LinearColor::RED;
                sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.4, 8, std::f32::consts::FRAC_PI_8);
                continue;
            }

            let size = 0.5; // light size

            sr.color = LinearColor::gray(0.3);
            sr.draw_rect_cos_sin(r_center, Z_SIGNAL, size + 0.1, size * 3.0 + 0.1, dir);

            for i in -1..2 {
                sr.draw_circle(r_center + i as f32 * dir_nor * size, Z_SIGNAL, size * 0.5);
            }
            sr.color = n.control.get_behavior(time).as_render_color().into();

            let offset = match n.control.get_behavior(time) {
                TrafficBehavior::RED => -size,
                TrafficBehavior::ORANGE => 0.0,
                TrafficBehavior::GREEN => size,
                _ => unreachable!(),
            };

            sr.draw_circle(r_center + offset * dir_nor, Z_SIGNAL, size * 0.5);
        }
    }

    fn arrows(&mut self, map: &Map, gfx: &GfxContext) -> Option<SpriteBatch> {
        self.arrow_builder.instances.clear();
        let lanes = map.lanes();
        for road in map.roads().values() {
            let fade = (road.length() - 5.0 - road.src_interface - road.dst_interface)
                .mul(0.2)
                .restrict(0.0, 1.0);

            let lanes = road
                .lanes_iter()
                .map(move |x| &lanes[*x])
                .filter(|l| l.kind.vehicles());
            for lane in lanes {
                for w in lane.points.as_slice().windows(2) {
                    let a = w[0];
                    let b = w[1];
                    let (dir, _) = match (b - a).dir_dist() {
                        Some(x) => x,
                        None => continue,
                    };
                    let mid = w[0] * 0.5 + w[1] * 0.5;
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

                    let dir: Vector2<f32> = (to - from) / l;

                    let t = Transform::new_cos_sin(from + dir * 2.25, dir);
                    let mut m = t.to_matrix4(Z_CROSSWALK);

                    m.x.x *= l - 4.0;
                    m.x.y *= l - 4.0;

                    m.y.x *= 3.0;
                    m.y.y *= 3.0;

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
        if map.dirty || self.road_mesh.is_none() {
            self.road_mesh = Self::road_mesh(map, Tesselator::new(None, 15.0), &ctx.gfx);
            self.arrows = self.arrows(map, &ctx.gfx);
            self.crosswalks = self.crosswalks(map, &ctx.gfx);

            map.dirty = false;
        }

        if let Some(x) = self.road_mesh.clone() {
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
