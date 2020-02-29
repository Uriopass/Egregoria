use crate::rendering::meshrenderable::scale_color;
use crate::rendering::render_context::RenderContext;
use cgmath::{InnerSpace, Vector2};
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
    pub fn render(&mut self, map: &Map, time: u64, rc: &mut RenderContext) {
        let navmesh = &map.navmesh;

        rc.sr.color = WHITE;
        for (id, n) in navmesh {
            rc.sr.draw_circle(n.pos, 4.25);
            for e in navmesh.get_neighs(id) {
                let p2 = navmesh.get(e.to).unwrap().pos;
                rc.sr.draw_stroke(n.pos, p2, 8.5);
            }
        }

        rc.sr.color = MID_GRAY;
        for (id, n) in navmesh {
            rc.sr.draw_circle(n.pos, 3.75);

            for e in navmesh.get_neighs(id) {
                let p2 = navmesh.get(e.to).unwrap().pos;
                rc.sr.draw_stroke(n.pos, p2, 7.5);
            }
        }

        // draw traffic lights
        for (id, n) in navmesh {
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
        }
    }
}
