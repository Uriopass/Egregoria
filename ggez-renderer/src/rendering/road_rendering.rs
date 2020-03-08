use crate::rendering::render_context::RenderContext;
use ggez::graphics::{Color, WHITE};
use scale::map_model::{Map, TrafficBehavior};

pub struct RoadRenderer;
const MID_GRAY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
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
                let mut p = Vec::with_capacity(2 + turn.points.len());
                p.push(lanes[id.src].get_inter_node_pos(inter_id));
                p.extend_from_slice(&turn.points);
                p.push(lanes[id.dst].get_inter_node_pos(inter_id));

                rc.sr.draw_polyline(&p, 8.5);
            }
        }

        for (_, n) in lanes {
            rc.sr.draw_polyline(&n.points, 8.5);
            rc.sr.draw_circle(*n.points.first().unwrap(), 4.25);
            rc.sr.draw_circle(*n.points.last().unwrap(), 4.25);
        }

        rc.sr.color = MID_GRAY;
        for (inter_id, inter) in inters {
            for (id, turn) in &inter.turns {
                let mut p = Vec::with_capacity(2 + turn.points.len());
                p.push(lanes[id.src].get_inter_node_pos(inter_id));
                p.extend_from_slice(&turn.points);
                p.push(lanes[id.dst].get_inter_node_pos(inter_id));

                rc.sr.draw_polyline(&p, 7.5);
            }
        }

        for (_, n) in lanes {
            rc.sr.draw_polyline(&n.points, 7.5);
            rc.sr.draw_circle(*n.points.first().unwrap(), 3.75);
            rc.sr.draw_circle(*n.points.last().unwrap(), 3.75);
        }

        // draw traffic lights
        /*
        for (id, n) in to_iter {
            if n.control.is_always() {
                continue;
            }

            let id = navmesh.get_backward_neighs(id).first().map(|x| x.to);
            if id.is_none() {
                rc.sr.color = scale_color(scale::rendering::RED);
                rc.sr.color.a = 0.5;
                rc.sr.draw_rect_centered(n.pos, 20.0, 20.0);
                continue;
            }
            let id = id.unwrap();
            let dir = (navmesh[id].pos - n.pos).normalize();

            let dir_nor: Vector2<f32> = [-dir.y, dir.x].into();

            let r_center = n.pos + dir_nor * 2.0;

            if n.control.is_stop() {
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
        }*/
    }

    pub fn far_render(&mut self, map: &Map, _time: u64, rc: &mut RenderContext) {
        let inters = map.intersections();

        rc.sr.color = MID_GRAY;
        for (_, n) in inters {
            rc.sr.draw_circle(n.pos, 8.0);
        }

        for (_, n) in map.roads() {
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
