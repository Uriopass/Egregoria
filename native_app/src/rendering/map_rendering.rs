use crate::rendering::map_mesh::MapMeshHandler;
use common::Z_SIGNAL;
use egregoria::Egregoria;
use flat_spatial::storage::Storage;
use geom::{Color, LinearColor, AABB};
use map_model::{Lane, Map, ProjectFilter, ProjectKind, TrafficBehavior};
use std::sync::Arc;
use wgpu_engine::objload::obj_to_mesh;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedMesh, InstancedMeshBuilder, MeshInstance, Tesselator,
};

pub struct RoadRenderer {
    meshb: MapMeshHandler,

    trees: Option<InstancedMesh>,
    trees_builder: InstancedMeshBuilder,
    trees_dirt_id: u32,
    last_cam: AABB,
}

impl RoadRenderer {
    pub fn new(gfx: &mut GfxContext, goria: &Egregoria) -> Self {
        RoadRenderer {
            meshb: MapMeshHandler::new(gfx, goria),
            last_cam: AABB::zero(),
            trees: None,
            trees_builder: InstancedMeshBuilder::new(Arc::new(
                obj_to_mesh("assets/pine.obj", gfx, gfx.palette()).expect("could not load pine"),
            )),
            trees_dirt_id: 0,
        }
    }

    fn render_lane_signals(n: &Lane, sr: &mut Tesselator, time: u32) {
        if n.control.is_always() {
            return;
        }

        let dir = n.orientation_from(n.dst);
        let dir_perp = dir.perpendicular();

        let r_center = n.points.last() + dir_perp * -3.5 + dir * -1.0;

        // Stop sign
        if n.control.is_stop_sign() {
            sr.set_color(LinearColor::WHITE);
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.5, 8, std::f32::consts::FRAC_PI_8);

            sr.set_color(LinearColor::RED);
            sr.draw_regular_polygon(r_center, Z_SIGNAL, 0.4, 8, std::f32::consts::FRAC_PI_8);
            return;
        }

        // Traffic light
        let size = 0.5; // light size

        sr.color = Color::gray(0.2).into();
        sr.draw_rect_cos_sin(r_center, Z_SIGNAL, size + 0.1, size * 3.0 + 0.1, dir);

        for i in -1..2 {
            sr.draw_circle(r_center + i as f32 * dir_perp * size, Z_SIGNAL, size * 0.5);
        }
        sr.set_color(match n.control.get_behavior(time) {
            TrafficBehavior::RED | TrafficBehavior::STOP => LinearColor::RED,
            TrafficBehavior::ORANGE => LinearColor::ORANGE,
            TrafficBehavior::GREEN => LinearColor::GREEN,
        });

        let offset = match n.control.get_behavior(time) {
            TrafficBehavior::RED => -size,
            TrafficBehavior::ORANGE => 0.0,
            TrafficBehavior::GREEN => size,
            TrafficBehavior::STOP => unreachable!(),
        };

        sr.draw_circle(r_center + offset * dir_perp, Z_SIGNAL, size * 0.5);
    }

    fn signals_render(map: &Map, time: u32, sr: &mut Tesselator) {
        match sr.cull_rect {
            Some(rect) => {
                if rect.w().max(rect.h()) > 1500.0 {
                    return;
                }
                for n in map
                    .spatial_map()
                    .query(rect, ProjectFilter::ROAD)
                    .filter_map(|k| match k {
                        ProjectKind::Road(id) => Some(id),
                        _ => None,
                    })
                    .flat_map(|id| map.roads()[id].lanes_iter())
                    .map(|(id, _)| &map.lanes()[id])
                {
                    Self::render_lane_signals(n, sr, time);
                }
            }
            None => {
                for n in map.lanes().values() {
                    Self::render_lane_signals(n, sr, time);
                }
            }
        }
    }

    pub fn trees(&mut self, map: &Map, screen: AABB, gfx: &GfxContext) -> Option<InstancedMesh> {
        if map.trees.dirt_id.0 == self.trees_dirt_id {
            if let Some(trees) = self.trees.as_ref() {
                return Some(trees.clone());
            }
        }

        self.trees_dirt_id = map.trees.dirt_id.0;

        self.trees_builder.instances.clear();
        for h in map.trees.grid.handles() {
            let (pos, t) = map.trees.grid.get(h).unwrap();

            self.trees_builder.instances.push(MeshInstance {
                pos: pos.z(0.0),
                dir: t.dir.z(0.0) * t.size * 0.2,
                tint: ((1.0 - t.size * 0.05) * t.col * LinearColor::WHITE).a(1.0),
            });
        }

        self.trees_builder.build(gfx)
    }

    pub fn render(&mut self, map: &Map, time: u32, tess: &mut Tesselator, ctx: &mut FrameContext) {
        let screen = tess
            .cull_rect
            .expect("no cull rectangle, might render far too many trees");

        self.trees = self.trees(map, screen, ctx.gfx);

        if let Some(x) = self.meshb.latest_mesh(map, ctx.gfx).clone() {
            ctx.draw(x);
        }

        if let Some(x) = self.trees.clone() {
            ctx.draw(x);
        }

        Self::signals_render(map, time, tess);

        self.last_cam = screen;
    }
}
