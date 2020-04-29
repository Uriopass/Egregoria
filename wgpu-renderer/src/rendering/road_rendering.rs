use crate::engine::{Drawable, FrameContext, Mesh};
use crate::geometry::Tesselator;
use crate::rendering::CameraHandler;
use cgmath::{vec2, InnerSpace, Vector2};
use scale::map_model::{LaneKind, Map, TrafficBehavior, TurnKind};
use scale::rendering::LinearColor;

pub struct RoadRenderer {
    road_mesh: Option<Mesh>,
}

impl RoadRenderer {
    pub fn new() -> Self {
        RoadRenderer { road_mesh: None }
    }

    pub fn near_render(&mut self, map: &Map, sr: &mut Tesselator) {
        let mid_gray: LinearColor = LinearColor::gray(0.5);
        let high_gray: LinearColor = LinearColor::gray(0.7);

        let inters = map.intersections();
        let lanes = map.lanes();

        sr.color = LinearColor::WHITE;

        let mut p = Vec::with_capacity(8);

        for n in lanes.values() {
            sr.color = LinearColor::WHITE;

            let first = n.points.first().unwrap();
            let last = n.points.last().unwrap();

            let w = n.width + 0.5;
            sr.draw_stroke(first, last, 0.1, w);

            sr.color = match n.kind {
                LaneKind::Walking => high_gray,
                _ => mid_gray,
            };
            sr.draw_stroke(first, last, 0.2, n.width - 0.5);
        }

        for (inter_id, inter) in inters {
            for (id, turn) in &inter.turns {
                sr.color = LinearColor::WHITE;

                if let TurnKind::Crosswalk = turn.kind {
                    let from = lanes[id.src].get_inter_node_pos(inter_id);
                    let to = lanes[id.dst].get_inter_node_pos(inter_id);

                    let l = (to - from).magnitude();

                    let dir: Vector2<f32> = (to - from) / l;
                    let normal = vec2(-dir.y, dir.x);
                    for i in 2..l as usize - 1 {
                        let along = from + dir * i as f32;
                        sr.draw_stroke(along - normal * 1.5, along + normal * 1.5, 0.21, 0.5);
                    }
                    continue;
                }

                let w = lanes[id.src].width;

                let first_dir = lanes[id.src].get_orientation_vec();
                let last_dir = lanes[id.dst].get_orientation_vec();

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                sr.draw_polyline_with_dir(&p, first_dir, last_dir, 0.1, w + 0.5);

                sr.color = match turn.kind {
                    TurnKind::Crosswalk => unreachable!(),
                    TurnKind::WalkingCorner => high_gray,
                    TurnKind::Normal => mid_gray,
                };

                p.clear();
                p.extend_from_slice(turn.points.as_slice());

                sr.draw_polyline_with_dir(&p, first_dir, last_dir, 0.2, w - 0.5);
            }
        }
    }

    pub fn signals_render(&mut self, map: &Map, time: u64, sr: &mut Tesselator) {
        for n in map.lanes().values() {
            if n.control.is_always() {
                continue;
            }

            let dir = n.get_orientation_vec();

            let dir_nor = vec2(dir.y, -dir.x);

            let r_center = n.points.last().unwrap() + dir_nor * 2.5 + dir * 2.5;

            if n.control.is_stop_sign() {
                sr.color = LinearColor::WHITE;
                sr.draw_regular_polygon(r_center, 0.3, 0.5, 8, std::f32::consts::FRAC_PI_8);

                sr.color = LinearColor::RED;
                sr.draw_regular_polygon(r_center, 0.3, 0.4, 8, std::f32::consts::FRAC_PI_8);
                continue;
            }

            let size = 0.5; // light size

            sr.color = LinearColor::gray(0.3);
            sr.draw_rect_cos_sin(r_center, 0.3, size + 0.1, size * 3.0 + 0.1, dir);

            for i in -1..2 {
                sr.draw_circle(r_center + i as f32 * dir_nor * size, 0.3, size * 0.5);
            }
            sr.color = n.control.get_behavior(time).as_render_color().into();

            let offset = match n.control.get_behavior(time) {
                TrafficBehavior::RED => -size,
                TrafficBehavior::ORANGE => 0.0,
                TrafficBehavior::GREEN => size,
                _ => unreachable!(),
            };

            sr.draw_circle(r_center + offset * dir_nor, 0.3, size * 0.5);
        }
    }

    pub fn far_render(&mut self, map: &Map, sr: &mut Tesselator) {
        let inters = map.intersections();

        sr.color = LinearColor::gray(0.5);
        for n in inters.values() {
            sr.draw_circle(n.pos, 0.1, 8.0);
        }

        for n in map.roads().values() {
            let pos1 = inters[n.src].pos;
            let pos2 = inters[n.dst].pos;

            sr.draw_stroke(
                pos1,
                pos2,
                0.1,
                n.lanes_iter().map(|x| map.lanes()[*x].width).sum(),
            );
        }
    }

    pub fn render(
        &mut self,
        map: &Map,
        time: u64,
        tess: &mut Tesselator,
        cam: &CameraHandler,
        ctx: &mut FrameContext,
        map_dirty: bool,
    ) {
        let render_near = cam.camera.zoom >= 1.5 || map.roads().len() < 1000;
        if map_dirty || self.road_mesh.is_none() {
            let mut tess = Tesselator::new(None, cam.camera.zoom);

            if render_near {
                self.near_render(map, &mut tess);
            } else {
                self.far_render(map, &mut tess);
            }

            self.road_mesh = tess.meshbuilder.build(ctx.gfx)
        }

        self.road_mesh.as_ref().unwrap().draw(ctx);

        self.signals_render(map, time, tess);
    }
}
