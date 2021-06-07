use egregoria::map_dynamic::Itinerary;
use egregoria::pedestrians::{Location, Pedestrian};
use egregoria::vehicles::{Vehicle, VehicleKind};
use egregoria::Egregoria;
use geom::{LinearColor, Transform, Vec3, V3};
use legion::query::*;
use wgpu_engine::meshload::load_mesh;
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
                gfx.texture("assets/path_not_found.png", "path_not_found"),
            ),
            cars: InstancedMeshBuilder::new(load_mesh("assets/simple_car.glb", gfx).unwrap()),
            trucks: InstancedMeshBuilder::new(load_mesh("assets/truck.glb", gfx).unwrap()),
            pedestrians: InstancedMeshBuilder::new(
                load_mesh("assets/pedestrian.glb", gfx).unwrap(),
            ),
        }
    }

    #[profiling::function]
    pub fn render(&mut self, goria: &Egregoria, fctx: &mut FrameContext) {
        self.cars.instances.clear();
        self.trucks.instances.clear();
        self.pedestrians.instances.clear();
        for (trans, v) in <(&Transform, &Vehicle)>::query().iter(goria.world()) {
            let v: &Vehicle = v;

            let instance = MeshInstance {
                pos: trans.position,
                dir: trans.dir,
                tint: v.tint.into(),
            };

            match v.kind {
                VehicleKind::Car => self.cars.instances.push(instance),
                VehicleKind::Truck => self.trucks.instances.push(instance),
                _ => {}
            }
        }

        for (trans, ped, loc) in <(&Transform, &Pedestrian, &Location)>::query().iter(goria.world())
        {
            let ped: &Pedestrian = ped;
            if matches!(loc, Location::Outside) {
                self.pedestrians.instances.push(MeshInstance {
                    pos: trans.position.up(0.5 + 0.4 * ped.walk_anim.cos()),
                    dir: trans.dir.xy().z0(),
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
                    trans.position + off * 3.0 * V3::Y + 3.0 * V3::Z,
                    Vec3::X,
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
