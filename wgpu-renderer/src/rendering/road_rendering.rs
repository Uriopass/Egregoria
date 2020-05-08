use crate::engine::{
    Drawable, FrameContext, GfxContext, InstanceRaw, Mesh, SpriteBatch, SpriteBatchBuilder, Texture,
};
use crate::geometry::Tesselator;
use cgmath::{vec2, InnerSpace, Vector2};
use scale::geometry::Vec2Impl;
use scale::map_model::{LaneKind, Map, TrafficBehavior, TurnKind};
use scale::physics::Transform;
use scale::rendering::LinearColor;

pub struct RoadRenderer {
    road_mesh: Option<Mesh>,
    arrows: Option<SpriteBatch>,
    arrow_builder: SpriteBatchBuilder,
}

const Z_LANE_BG: f32 = 0.21;
const Z_LANE: f32 = 0.22;
const Z_ARROW: f32 = 0.23;
const Z_CROSSWALK: f32 = 0.24;
const Z_SIGNAL: f32 = 0.25;

impl RoadRenderer {
    pub fn new(gfx: &GfxContext) -> Self {
        let arrow_builder = SpriteBatchBuilder::new(
            Texture::from_path(gfx, "resources/arrow_one_way.png", Some("arrow")).unwrap(),
        );
        RoadRenderer {
            road_mesh: None,
            arrows: None,
            arrow_builder,
        }
    }

    pub fn road_mesh(map: &Map, mut tess: Tesselator, gfx: &GfxContext) -> Option<Mesh> {
        let mid_gray: LinearColor = LinearColor::gray(0.5);
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
            tess.draw_stroke(first, last, Z_LANE, n.width - 0.5);
        }

        for (inter_id, inter) in inters {
            if inter.roads.is_empty() {
                tess.color = LinearColor::WHITE;
                tess.draw_circle(inter.pos, Z_LANE_BG, 5.5);

                tess.color = mid_gray;
                tess.draw_circle(inter.pos, Z_LANE, 5.0);
            }
            for turn in inter.turns() {
                tess.color = LinearColor::WHITE;
                let id = turn.id;

                if matches!(turn.kind, TurnKind::Crosswalk) {
                    let from = lanes[id.src].get_inter_node_pos(inter_id);
                    let to = lanes[id.dst].get_inter_node_pos(inter_id);

                    let l = (to - from).magnitude();

                    let dir: Vector2<f32> = (to - from) / l;
                    let normal = vec2(-dir.y, dir.x);
                    for i in 2..l as usize - 1 {
                        let along = from + dir * i as f32;
                        tess.draw_stroke(
                            along - normal * 1.5,
                            along + normal * 1.5,
                            Z_CROSSWALK,
                            0.5,
                        );
                    }
                    continue;
                }

                let w = lanes[id.src].width;

                let first_dir = lanes[id.src].get_orientation_vec();
                let last_dir = lanes[id.dst].get_orientation_vec();

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, Z_LANE_BG, w + 0.5);

                tess.color = match turn.kind {
                    TurnKind::Crosswalk => unreachable!(),
                    TurnKind::WalkingCorner => high_gray,
                    TurnKind::Driving => mid_gray,
                };

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                tess.draw_polyline_with_dir(&p, first_dir, last_dir, Z_LANE, w - 0.5);
            }
        }
        tess.meshbuilder.build(gfx)
    }

    pub fn signals_render(map: &Map, time: u64, sr: &mut Tesselator) {
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

    pub fn arrows(&mut self, map: &Map, gfx: &GfxContext) -> Option<SpriteBatch> {
        self.arrow_builder.instances.clear();
        let lanes = map.lanes();
        for road in map.roads().values() {
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
                        [0.7; 3],
                        4.0,
                    ));
                }
            }
        }
        self.arrow_builder.build(gfx)
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
            map.dirty = false;
        }

        if let Some(ref x) = self.road_mesh {
            x.draw(ctx)
        }

        if let Some(ref x) = self.arrows {
            x.draw(ctx)
        }

        Self::signals_render(map, time, tess);
    }
}
