use crate::geometry::tesselator::Tesselator;
use crate::rendering::meshrenderable::scale_color;
use crate::rendering::render_context::RenderContext;
use cgmath::{vec2, InnerSpace, Vector2};
use ggez::graphics::{Color, DrawParam, Mesh, WHITE};
use ggez::GameResult;
use scale::map_model::{LaneKind, Map, TrafficBehavior, TurnKind};

pub struct RoadRenderer {
    road_mesh: Option<Mesh>,
}
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
        RoadRenderer { road_mesh: None }
    }

    pub fn near_render(&mut self, map: &Map, sr: &mut Tesselator) {
        let inters = map.intersections();
        let lanes = map.lanes();

        sr.color = WHITE;

        let mut p = Vec::with_capacity(8);
        for (_, inter) in inters {
            for (id, turn) in &inter.turns {
                p.clear();
                p.push(turn.points[0] - lanes[id.src].get_orientation_vec());
                p.extend_from_slice(turn.points.as_slice());
                p.push(turn.points.last().unwrap() + lanes[id.dst].get_orientation_vec());

                sr.draw_polyline(&p, lanes[id.src].width + 0.5);
            }
        }

        for n in lanes.values() {
            let w = n.width + 0.5;
            sr.draw_polyline(n.points.as_slice(), w);
        }

        sr.color = MID_GRAY;
        for n in lanes.values() {
            sr.color = match n.kind {
                LaneKind::Walking => HIGH_GRAY,
                _ => MID_GRAY,
            };

            sr.draw_polyline(n.points.as_slice(), n.width - 0.5);
        }
        for (inter_id, inter) in inters {
            // Draw normal turns
            sr.color = MID_GRAY;
            for (id, turn) in &inter.turns {
                if turn.kind != TurnKind::Normal {
                    continue;
                }
                p.clear();
                p.push(turn.points[0] - lanes[id.src].get_orientation_vec());
                p.extend_from_slice(turn.points.as_slice());
                p.push(turn.points.last().unwrap() + lanes[id.dst].get_orientation_vec());

                sr.draw_polyline(&p, lanes[id.src].width - 0.5);
            }

            // Draw walking corners
            sr.color = HIGH_GRAY;
            for (id, turn) in &inter.turns {
                if turn.kind != TurnKind::WalkingCorner {
                    continue;
                }
                p.clear();
                p.push(turn.points[0] - lanes[id.src].get_orientation_vec());
                p.extend_from_slice(turn.points.as_slice());
                p.push(turn.points.last().unwrap() + lanes[id.dst].get_orientation_vec());

                sr.draw_polyline(&p, lanes[id.src].width - 0.5);
            }

            // Draw crosswalks
            sr.color = WHITE;
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
                    sr.draw_stroke(along - normal * 1.5, along + normal * 1.5, 0.5);
                }
            }
        }
    }

    pub fn signals_render(&mut self, map: &Map, time: u64, sr: &mut Tesselator) {
        for n in map.lanes().values() {
            if n.control.is_always() {
                continue;
            }

            let dir = n.get_orientation_vec();

            let dir_nor = vec2(-dir.y, dir.x);

            let r_center = n.points.last().unwrap() + dir_nor * 2.0 + dir * 2.5;

            if n.control.is_stop() {
                sr.color = scale_color(scale::rendering::Color::WHITE);
                sr.draw_rect_cos_sin(
                    r_center,
                    1.5,
                    1.5,
                    vec2(
                        std::f32::consts::FRAC_1_SQRT_2,
                        std::f32::consts::FRAC_1_SQRT_2,
                    ),
                );

                sr.color = scale_color(scale::rendering::Color::RED);
                sr.draw_rect_cos_sin(
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

            sr.color = scale_color(scale::rendering::Color::gray(0.3));
            sr.draw_rect_cos_sin(r_center, 1.1, 3.1, dir);

            sr.color = scale_color(scale::rendering::Color::gray(0.1));
            for i in -1..2 {
                sr.draw_circle(r_center + i as f32 * dir_nor, 0.5);
            }
            sr.color = scale_color(n.control.get_behavior(time).as_render_color());

            let offset = match n.control.get_behavior(time) {
                TrafficBehavior::RED => -1.0,
                TrafficBehavior::ORANGE => 0.0,
                TrafficBehavior::GREEN => 1.0,
                _ => unreachable!(),
            };

            sr.draw_circle(r_center + offset * dir_nor, 0.5);
        }
    }

    pub fn far_render(&mut self, map: &Map, sr: &mut Tesselator) {
        let inters = map.intersections();

        sr.color = MID_GRAY;
        for n in inters.values() {
            sr.draw_circle(n.pos, 8.0);
        }

        for n in map.roads().values() {
            let pos1 = inters[n.src].pos;
            let pos2 = inters[n.dst].pos;

            sr.draw_stroke(
                pos1,
                pos2,
                n.lanes_iter().map(|x| map.lanes()[*x].width).sum(),
            );
        }
    }

    pub fn render(
        &mut self,
        map: &Map,
        time: u64,
        rc: &mut RenderContext,
        map_dirty: bool,
    ) -> GameResult<()> {
        let render_near = rc.cam.camera.zoom >= 1.5 || map.roads().len() < 1000;
        if map_dirty || self.road_mesh.is_none() {
            let mut tess = Tesselator::new(rc.cam.get_screen_box(), rc.cam.camera.zoom, false);

            if render_near {
                self.near_render(map, &mut tess);
            } else {
                self.far_render(map, &mut tess);
            }

            self.road_mesh = tess.meshbuilder.build(rc.ctx).ok()
        }

        ggez::graphics::draw(
            rc.ctx,
            self.road_mesh.as_ref().unwrap(),
            DrawParam::default(),
        )?;

        if render_near {
            self.signals_render(map, time, &mut rc.tess);
        }
        Ok(())
    }
}
