use crate::rendering::meshrenderable::scale_color;
use crate::rendering::render_context::RenderContext;
use ggez::graphics::{Color, WHITE};
use scale::map_model::Map;

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

        for (_, n) in navmesh {
            if n.light.is_always() {
                continue;
            }
            rc.sr.color = scale_color(n.light.get_color(time).as_render_color());
            rc.sr.draw_circle(n.pos, 0.5);
        }
    }
}
