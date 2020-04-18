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

        let mut p = Vec::with_capacity(8);
        for (_, inter) in inters {
            for (id, turn) in &inter.turns {
                p.clear();
                p.push(turn.points[0] - lanes[id.src].get_orientation_vec());
                p.extend_from_slice(turn.points.as_slice());
                p.push(turn.points.last().unwrap() + lanes[id.dst].get_orientation_vec());

                rc.sr.draw_polyline(&p, lanes[id.src].width + 0.5);
            }
        }

        for n in lanes.values() {
            let w = n.width + 0.5;
            rc.sr.draw_polyline(n.points.as_slice(), w);
        }

        rc.sr.color = MID_GRAY;
        for n in lanes.values() {
            rc.sr.color = match n.kind {
                LaneKind::Walking => HIGH_GRAY,
                _ => MID_GRAY,
            };

            rc.sr.draw_polyline(n.points.as_slice(), n.width - 0.5);
        }
        for (inter_id, inter) in inters {
            // Draw normal turns
            rc.sr.color = MID_GRAY;
            for (id, turn) in &inter.turns {
                if turn.kind != TurnKind::Normal {
                    continue;
                }
                p.clear();
                p.push(turn.points[0] - lanes[id.src].get_orientation_vec());
                p.extend_from_slice(turn.points.as_slice());
                p.push(turn.points.last().unwrap() + lanes[id.dst].get_orientation_vec());

                rc.sr.draw_polyline(&p, lanes[id.src].width - 0.5);
            }

            // Draw walking corners
            rc.sr.color = HIGH_GRAY;
            for (id, turn) in &inter.turns {
                if turn.kind != TurnKind::WalkingCorner {
                    continue;
                }
                p.clear();
                p.push(turn.points[0] - lanes[id.src].get_orientation_vec());
                p.extend_from_slice(turn.points.as_slice());
                p.push(turn.points.last().unwrap() + lanes[id.dst].get_orientation_vec());

                rc.sr.draw_polyline(&p, lanes[id.src].width - 0.5);
            }

            // Draw crosswalks
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
                for i in 2..l as usize - 1 {
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
                rc.sr.color = scale_color(scale::rendering::Color::WHITE);
                rc.sr.draw_rect_cos_sin(
                    r_center,
                    1.5,
                    1.5,
                    vec2(
                        std::f32::consts::FRAC_1_SQRT_2,
                        std::f32::consts::FRAC_1_SQRT_2,
                    ),
                );

                rc.sr.color = scale_color(scale::rendering::Color::RED);
                rc.sr.draw_rect_cos_sin(
                    r_center,
                    1.0,
                    1.0,
                    vec2(
                        std::f32::consts::FRAC_1_SQRT_2,
                        std::f32::consts::FRAC_1_SQRT_2,
                    ),
                );
                continue;
            }

            rc.sr.color = scale_color(scale::rendering::Color::gray(0.3));
            rc.sr.draw_rect_cos_sin(r_center, 1.1, 3.1, dir);

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

            rc.sr.draw_stroke(
                pos1,
                pos2,
                n.lanes_iter().map(|x| map.lanes()[*x].width).sum(),
            );
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
