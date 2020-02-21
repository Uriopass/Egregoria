use crate::rendering::meshrenderable::scale_color;
use crate::rendering::render_context::RenderContext;
use cgmath::{InnerSpace, Vector2};
use ggez::graphics::{Color, WHITE};
use scale::map_model::{Map, TrafficLightColor};

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

        for (id, n) in navmesh {
            rc.sr.color = WHITE;
            rc.sr.draw_circle(n.pos, 4.25);
            for e in navmesh.get_neighs(id) {
                let p2 = navmesh.get(e.to).unwrap().pos;
                rc.sr.draw_stroke(n.pos, p2, 8.5);
            }
        }

        for (id, n) in navmesh {
            rc.sr.color = MID_GRAY;
            rc.sr.draw_circle(n.pos, 3.75);

            for e in navmesh.get_neighs(id) {
                let p2 = navmesh.get(e.to).unwrap().pos;
                rc.sr.draw_stroke(n.pos, p2, 7.5);
            }
        }

        // draw traffic lights
        for (id, n) in navmesh {
            if n.light.is_always() {
                continue;
            }

            let dir = (navmesh[navmesh.get_backward_neighs(id).first().unwrap().to].pos - n.pos)
                .normalize();

            let dir_nor: Vector2<f32> = [-dir.y, dir.x].into();

            let r_center = n.pos + dir_nor * 2.0;
            rc.sr.color = scale_color(scale::rendering::Color::gray(0.3));
            rc.sr.draw_rect_cos_sin(r_center, 1.1, 3.1, dir.x, dir.y);

            rc.sr.color = scale_color(scale::rendering::Color::gray(0.1));
            for i in -1..2 {
                rc.sr.draw_circle(r_center + i as f32 * dir_nor, 0.5);
            }
            rc.sr.color = scale_color(n.light.get_color(time).as_render_color());

            let offset = match n.light.get_color(time) {
                TrafficLightColor::RED => -1.0,
                TrafficLightColor::ORANGE(_) => 0.0,
                TrafficLightColor::GREEN => 1.0,
            };

            rc.sr.draw_circle(r_center + offset * dir_nor, 0.5);
        }
    }
}
