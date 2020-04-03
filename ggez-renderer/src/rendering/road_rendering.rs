use crate::rendering::meshrenderable::scale_color;
use crate::rendering::render_context::RenderContext;
use cgmath::{vec2, InnerSpace, Vector2};
use ggez::graphics::{Color, WHITE};
use scale::map_model::{LaneKind, Map, TrafficBehavior, TurnKind};

pub struct RoadRenderer;
const MID_GRAY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

const HIGH_GRAY: Color = Color {
    r: 0.7,
    g: 0.7,
    b: 0.7,
    a: 1.0,
};

impl RoadRenderer {
    pub fn new() -> Self {
        RoadRenderer
    }

    pub fn near_render(&mut self, map: &Map, time: u64, rc: &mut RenderContext) {
        let inters = map.intersections();
        let lanes = map.lanes();

        rc.sr.color = WHITE;

        for (inter_id, inter) in inters {
            for (id, turn) in &inter.turns {
                let mut p = Vec::with_capacity(2 + turn.points.n_points());
                p.push(lanes[id.src].get_inter_node_pos(inter_id));
                p.extend_from_slice(turn.points.as_slice());
                p.push(lanes[id.dst].get_inter_node_pos(inter_id));

                rc.sr.draw_polyline(&p, 8.5);
            }
        }

        for n in lanes.values() {
            rc.sr.draw_polyline(n.points.as_slice(), 8.5);
            rc.sr.draw_circle(*n.points.first().unwrap(), 4.25);
            rc.sr.draw_circle(*n.points.last().unwrap(), 4.25);
        }

        rc.sr.color = MID_GRAY;
        for n in lanes.values() {
            rc.sr.color = match n.kind {
                LaneKind::Walking => HIGH_GRAY,
                _ => MID_GRAY,
            };

            rc.sr.draw_polyline(n.points.as_slice(), 7.5);
            rc.sr.draw_circle(*n.points.first().unwrap(), 3.75);
            rc.sr.draw_circle(*n.points.last().unwrap(), 3.75);

            rc.sr.color = WHITE;
            if n.control.is_stop() || n.control.is_light() {
                if let [.., b, p] = n.points.as_slice() {
                    let dir = (p - b).normalize();
                    let dir_nor: Vector2<f32> = [-dir.y, dir.x].into();
                    rc.sr.draw_stroke(
                        p + dir * 1.5 + 4.0 * dir_nor,
                        p + dir * 1.5 - 4.0 * dir_nor,
                        0.5,
                    );
                }
            }
        }
        for (inter_id, inter) in inters {
            rc.sr.color = MID_GRAY;
            for (id, turn) in &inter.turns {
                if turn.kind != TurnKind::Normal {
                    continue;
                }
                let mut p = Vec::with_capacity(2 + turn.points.n_points());
                p.push(lanes[id.src].get_inter_node_pos(inter_id));
                p.extend_from_slice(turn.points.as_slice());
                p.push(lanes[id.dst].get_inter_node_pos(inter_id));

                rc.sr.draw_polyline(&p, 7.5);
            }

            rc.sr.color = HIGH_GRAY;
            for (id, turn) in &inter.turns {
                if turn.kind != TurnKind::WalkingCorner {
                    continue;
                }
                let mut p = Vec::with_capacity(2 + turn.points.n_points());
                p.push(lanes[id.src].get_inter_node_pos(inter_id));
                p.extend_from_slice(turn.points.as_slice());
                p.push(lanes[id.dst].get_inter_node_pos(inter_id));

                rc.sr.draw_polyline(&p, 7.5);
            }

            rc.sr.color = WHITE;
            for (id, turn) in &inter.turns {
                if turn.kind != TurnKind::Crosswalk {
                    continue;
                }

                let from = lanes[id.src].get_inter_node_pos(inter_id);
                let to = lanes[id.dst].get_inter_node_pos(inter_id);

                let l = (to - from).magnitude();

                let dir: Vector2<f32> = (to - from) / l;
                let normal = vec2(-dir.y, dir.x);

                for i in 4..l as usize - 3 {
                    let along = from + dir * i as f32;
                    rc.sr
                        .draw_stroke(along - normal * 1.5, along + normal * 1.5, 0.5);
                }
            }
        }

        // draw traffic lights

        for n in lanes.values() {
            if n.control.is_always() {
                continue;
            }

            let dir = n.get_orientation_vec();

            let dir_nor = vec2(-dir.y, dir.x);

            let r_center = n.points.last().unwrap() + dir_nor * 2.0 + dir * 2.5;

            if n.control.is_stop() {
                rc.sr.color = scale_color(scale::rendering::WHITE);
                rc.sr.draw_rect_cos_sin(
                    r_center,
                    1.5,
                    1.5,
                    std::f32::consts::FRAC_1_SQRT_2,
                    std::f32::consts::FRAC_1_SQRT_2,
                );

                rc.sr.color = scale_color(scale::rendering::RED);
                rc.sr.draw_rect_cos_sin(
                    r_center,
                    1.0,
                    1.0,
                    std::f32::consts::FRAC_1_SQRT_2,
                    std::f32::consts::FRAC_1_SQRT_2,
                );
                continue;
            }

            rc.sr.color = scale_color(scale::rendering::Color::gray(0.3));
            rc.sr.draw_rect_cos_sin(r_center, 1.1, 3.1, dir.x, dir.y);

            rc.sr.color = scale_color(scale::rendering::Color::gray(0.1));
            for i in -1..2 {
                rc.sr.draw_circle(r_center + i as f32 * dir_nor, 0.5);
            }
            rc.sr.color = scale_color(n.control.get_behavior(time).as_render_color());

            let offset = match n.control.get_behavior(time) {
                TrafficBehavior::RED => -1.0,
                TrafficBehavior::ORANGE => 0.0,
                TrafficBehavior::GREEN => 1.0,
                _ => unreachable!(),
            };

            rc.sr.draw_circle(r_center + offset * dir_nor, 0.5);
        }
    }

    pub fn far_render(&mut self, map: &Map, _time: u64, rc: &mut RenderContext) {
        let inters = map.intersections();

        rc.sr.color = MID_GRAY;
        for n in inters.values() {
            rc.sr.draw_circle(n.pos, 8.0);
        }

        for n in map.roads().values() {
            let pos1 = inters[n.src].pos;
            let pos2 = inters[n.dst].pos;

            rc.sr.draw_stroke(pos1, pos2, n.n_lanes() as f32 * 8.0);
        }
    }

    pub fn render(&mut self, map: &Map, time: u64, rc: &mut RenderContext) {
        if rc.cam.camera.zoom < 1.5 && map.roads().len() > 1000 {
            self.far_render(map, time, rc);
        } else {
            self.near_render(map, time, rc);
        }
    }
}
