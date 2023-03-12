use egregoria::map_dynamic::Itinerary;
use egregoria::transportation::train::{RailWagon, RailWagonKind};
use egregoria::transportation::{Location, Pedestrian, Vehicle, VehicleKind};
use egregoria::Egregoria;
use geom::{LinearColor, Transform, Vec3, V3};
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedMeshBuilder, MeshInstance, SpriteBatchBuilder,
};

pub(crate) struct InstancedRender {
    pub(crate) path_not_found: SpriteBatchBuilder,
    pub(crate) cars: InstancedMeshBuilder,
    pub(crate) locomotives: InstancedMeshBuilder,
    pub(crate) wagons_passenger: InstancedMeshBuilder,
    pub(crate) wagons_freight: InstancedMeshBuilder,
    pub(crate) trucks: InstancedMeshBuilder,
    pub(crate) pedestrians: InstancedMeshBuilder,
}

impl InstancedRender {
    pub(crate) fn new(gfx: &mut GfxContext) -> Self {
        defer!(log::info!("finished init of instanced render"));

        let car = load_mesh(gfx, "simple_car.glb").unwrap();
        InstancedRender {
            path_not_found: SpriteBatchBuilder::new(
                gfx.texture("assets/sprites/path_not_found.png", "path_not_found"),
                gfx,
            ),
            cars: InstancedMeshBuilder::new(car),
            locomotives: InstancedMeshBuilder::new(load_mesh(gfx, "train.glb").unwrap()),
            wagons_freight: InstancedMeshBuilder::new(load_mesh(gfx, "wagon_freight.glb").unwrap()),
            wagons_passenger: InstancedMeshBuilder::new(load_mesh(gfx, "wagon.glb").unwrap()),
            trucks: InstancedMeshBuilder::new(load_mesh(gfx, "truck.glb").unwrap()),
            pedestrians: InstancedMeshBuilder::new(load_mesh(gfx, "pedestrian.glb").unwrap()),
        }
    }

    #[profiling::function]
    pub(crate) fn render(&mut self, goria: &Egregoria, fctx: &mut FrameContext<'_>) {
        self.cars.instances.clear();
        self.trucks.instances.clear();
        self.pedestrians.instances.clear();
        for (_, (trans, v)) in goria.world().query::<(&Transform, &Vehicle)>().iter() {
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

        self.locomotives.instances.clear();
        self.wagons_passenger.instances.clear();
        self.wagons_freight.instances.clear();
        for (_, (trans, wagon)) in goria.world().query::<(&Transform, &RailWagon)>().iter() {
            let instance = MeshInstance {
                pos: trans.position,
                dir: trans.dir,
                tint: LinearColor::WHITE,
            };

            match wagon.kind {
                RailWagonKind::Passenger => {
                    self.wagons_passenger.instances.push(instance);
                }
                RailWagonKind::Freight => {
                    self.wagons_freight.instances.push(instance);
                }
                RailWagonKind::Locomotive => {
                    self.locomotives.instances.push(instance);
                }
            }
        }

        for (_, (trans, ped, loc)) in goria
            .world()
            .query::<(&Transform, &Pedestrian, &Location)>()
            .iter()
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
        for (_, (trans, itin)) in goria.world().query::<(&Transform, &Itinerary)>().iter() {
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
        if let Some(x) = self.locomotives.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
        if let Some(x) = self.wagons_passenger.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
        if let Some(x) = self.wagons_freight.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
    }
}
