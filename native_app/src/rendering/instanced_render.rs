use common::Z_PATH_NOT_FOUND;
use egregoria::map_dynamic::Itinerary;
use egregoria::pedestrians::{Location, Pedestrian};
use egregoria::rendering::assets::AssetRender;
use egregoria::Egregoria;
use geom::{LinearColor, Transform, Vec2};
use legion::query::*;
use std::sync::Arc;
use wgpu_engine::objload::obj_to_mesh;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedPaletteMeshBuilder, MeshInstance, SpriteBatchBuilder,
};

pub struct InstancedRender {
    pub texs: Vec<SpriteBatchBuilder>,
    pub path_not_found: SpriteBatchBuilder,
    pub cars: InstancedPaletteMeshBuilder,
    pub trucks: InstancedPaletteMeshBuilder,
    pub pedestrians: InstancedPaletteMeshBuilder,
}

impl InstancedRender {
    pub fn new(ctx: &mut GfxContext) -> Self {
        let car = ctx.texture("assets/car.png", Some("cartex"));
        let spr_car = SpriteBatchBuilder::new(car);

        let truck = ctx.texture("assets/truck.png", Some("trucktex"));
        let spr_truck = SpriteBatchBuilder::new(truck);

        let texs = vec![spr_car, spr_truck];
        InstancedRender {
            texs,
            path_not_found: SpriteBatchBuilder::new(
                ctx.texture("assets/path_not_found.png", Some("path_not_found")),
            ),
            cars: InstancedPaletteMeshBuilder::new(Arc::new(
                obj_to_mesh("assets/simple_car.obj", ctx).unwrap(),
            )),
            trucks: InstancedPaletteMeshBuilder::new(Arc::new(
                obj_to_mesh("assets/truck.obj", ctx).unwrap(),
            )),
            pedestrians: InstancedPaletteMeshBuilder::new(Arc::new(
                obj_to_mesh("assets/pedestrian.obj", ctx).unwrap(),
            )),
        }
    }

    pub fn render(&mut self, goria: &Egregoria, fctx: &mut FrameContext) {
        for x in &mut self.texs {
            x.clear();
        }

        self.cars.instances.clear();
        self.trucks.instances.clear();
        self.pedestrians.instances.clear();
        for (trans, ar) in <(&Transform, &AssetRender)>::query().iter(goria.world()) {
            let ar: &AssetRender = ar;
            if ar.hide {
                continue;
            }

            if ar.id.id == 0 {
                self.cars.instances.push(MeshInstance {
                    pos: trans.position().z(1.5),
                    dir: trans.direction().z(0.0),
                    tint: ar.tint.into(),
                });
            }

            if ar.id.id == 1 {
                self.trucks.instances.push(MeshInstance {
                    pos: trans.position().z(1.5),
                    dir: trans.direction().z(0.0),
                    tint: ar.tint.into(),
                });
            }

            self.texs[ar.id.id as usize].push(
                trans.position(),
                trans.direction(),
                ar.z,
                ar.tint.into(),
                (ar.scale, ar.scale),
            );
        }

        for (trans, ped, loc) in <(&Transform, &Pedestrian, &Location)>::query().iter(goria.world())
        {
            let ped: &Pedestrian = ped;
            if matches!(loc, Location::Outside) {
                self.pedestrians.instances.push(MeshInstance {
                    pos: trans.position().z(1.5 + 0.4 * ped.walk_anim.cos()),
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

        for x in &mut self.texs {
            if let Some(x) = x.build(fctx.gfx) {
                fctx.objs.push(Box::new(x));
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
