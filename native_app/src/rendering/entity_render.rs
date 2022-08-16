use egregoria::map_dynamic::Itinerary;
use egregoria::pedestrians::{Location, Pedestrian};
use egregoria::vehicles::trains::{Locomotive, RailWagon, RailWagonKind};
use egregoria::vehicles::{Vehicle, VehicleKind};
use egregoria::Egregoria;
use geom::{LinearColor, Transform, Vec3, V3};
use hecs::With;
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::{
    FrameContext, GfxContext, InstancedMeshBuilder, MeshInstance, SpriteBatchBuilder,
};

pub struct InstancedRender {
    pub path_not_found: SpriteBatchBuilder,
    pub cars: InstancedMeshBuilder,
    pub trains: InstancedMeshBuilder,
    pub wagons_passenger: InstancedMeshBuilder,
    pub wagons_fret: InstancedMeshBuilder,
    pub trucks: InstancedMeshBuilder,
    pub pedestrians: InstancedMeshBuilder,
}

impl InstancedRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        defer!(log::info!("finished init of instanced render"));
        InstancedRender {
            path_not_found: SpriteBatchBuilder::new(
                gfx.texture("assets/sprites/path_not_found.png", "path_not_found"),
            ),
            cars: InstancedMeshBuilder::new(load_mesh("simple_car.glb", gfx).unwrap()),
            trains: InstancedMeshBuilder::new(load_mesh("train.glb", gfx).unwrap()),
            wagons_fret: InstancedMeshBuilder::new(load_mesh("wagon_fret.glb", gfx).unwrap()),
            wagons_passenger: InstancedMeshBuilder::new(load_mesh("wagon.glb", gfx).unwrap()),
            trucks: InstancedMeshBuilder::new(load_mesh("truck.glb", gfx).unwrap()),
            pedestrians: InstancedMeshBuilder::new(load_mesh("pedestrian.glb", gfx).unwrap()),
        }
    }

    #[profiling::function]
    pub fn render(&mut self, goria: &Egregoria, fctx: &mut FrameContext<'_>) {
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

        self.trains.instances.clear();
        for (_, trans) in goria
            .world()
            .query::<With<&Transform, &Locomotive>>()
            .iter()
        {
            let instance = MeshInstance {
                pos: trans.position,
                dir: trans.dir,
                tint: LinearColor::WHITE,
            };
            self.trains.instances.push(instance);
        }

        self.wagons_passenger.instances.clear();
        self.wagons_fret.instances.clear();
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
                RailWagonKind::Fret => {
                    self.wagons_fret.instances.push(instance);
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
        if let Some(x) = self.trains.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
        if let Some(x) = self.wagons_passenger.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
        if let Some(x) = self.wagons_fret.build(fctx.gfx) {
            fctx.objs.push(Box::new(x));
        }
    }
}
