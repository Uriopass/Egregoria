use common::Z_PATH_NOT_FOUND;
use egregoria::map_dynamic::Itinerary;
use egregoria::pedestrians::{Location, Pedestrian};
use egregoria::rendering::assets::{AssetID, AssetRender};
use egregoria::Egregoria;
use geom::{LinearColor, Transform, Vec2};
use legion::query::*;
use std::sync::Arc;
use wgpu_engine::objload::obj_to_mesh;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedMeshBuilder, MeshInstance, SpriteBatchBuilder,
};

pub struct InstancedRender {
    pub path_not_found: SpriteBatchBuilder,
    pub cars: InstancedMeshBuilder,
    pub trucks: InstancedMeshBuilder,
    pub pedestrians: InstancedMeshBuilder,
}

impl InstancedRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        InstancedRender {
            path_not_found: SpriteBatchBuilder::new(
                gfx.texture("assets/path_not_found.png", Some("path_not_found")),
            ),
            cars: InstancedMeshBuilder::new(Arc::new(
                obj_to_mesh("assets/simple_car.obj", gfx, gfx.palette()).unwrap(),
            )),
            trucks: InstancedMeshBuilder::new(Arc::new(
                obj_to_mesh("assets/truck.obj", gfx, gfx.palette()).unwrap(),
            )),
            pedestrians: InstancedMeshBuilder::new(Arc::new(
                obj_to_mesh("assets/pedestrian.obj", gfx, gfx.palette()).unwrap(),
            )),
        }
    }

    pub fn render(&mut self, goria: &Egregoria, fctx: &mut FrameContext) {
        self.cars.instances.clear();
        self.trucks.instances.clear();
        self.pedestrians.instances.clear();
        for (trans, ar) in <(&Transform, &AssetRender)>::query().iter(goria.world()) {
            let ar: &AssetRender = ar;

            let instance = MeshInstance {
                pos: trans.position().z(0.5),
                dir: trans.direction().z(0.0),
                tint: ar.tint.into(),
            };

            match ar.id {
                AssetID::CAR => self.cars.instances.push(instance),
                AssetID::TRUCK => self.trucks.instances.push(instance),
                _ => {}
            }
        }

        for (trans, ped, loc) in <(&Transform, &Pedestrian, &Location)>::query().iter(goria.world())
        {
            let ped: &Pedestrian = ped;
            if matches!(loc, Location::Outside) {
                self.pedestrians.instances.push(MeshInstance {
                    pos: trans.position().z(0.5 + 0.4 * ped.walk_anim.cos()),
                    dir: trans.direction().z(0.0),
                    tint: LinearColor::WHITE,
                });
            }
        }

        self.path_not_found.clear();
        for (trans, itin) in <(&Transform, &Itinerary)>::query().iter(goria.world()) {
            let itin: &Itinerary = itin;
            if let Some(wait) = itin.is_wait_for_reroute() {
                if wait == 0 {
                    continue;
                }

                let r = wait as f32 / 200.0;
                let off = 1.0 - r;

                let s = 7.0;
                self.path_not_found.push(
                    trans.position() + off * 3.0 * Vec2::UNIT_Y,
                    Vec2::UNIT_X,
                    Z_PATH_NOT_FOUND,
                    LinearColor::RED.a(r),
                    (s, s),
                );
            }
        }

        if let Some(x) = self.path_not_found.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
        if let Some(x) = self.cars.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
        if let Some(x) = self.trucks.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
        if let Some(x) = self.pedestrians.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
    }
}
