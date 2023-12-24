use engine::meshload::load_mesh;
use engine::{FrameContext, GfxContext, InstancedMeshBuilder, MeshInstance, SpriteBatchBuilder};
use geom::{LinearColor, Vec3, V3};
use simulation::transportation::train::RailWagonKind;
use simulation::transportation::{Location, VehicleKind};
use simulation::Simulation;

/// Render all entities using instanced rendering for performance
pub struct InstancedRender {
    pub path_not_found: SpriteBatchBuilder<true>,
    pub cars: InstancedMeshBuilder<true>,
    pub locomotives: InstancedMeshBuilder<true>,
    pub wagons_passenger: InstancedMeshBuilder<true>,
    pub wagons_freight: InstancedMeshBuilder<true>,
    pub trucks: InstancedMeshBuilder<true>,
    pub pedestrians: InstancedMeshBuilder<true>,
    pub birds: InstancedMeshBuilder<true>,
}

impl InstancedRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
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
            birds: InstancedMeshBuilder::new(load_mesh(gfx, "bird.glb").unwrap()),
        }
    }

    pub fn render(&mut self, sim: &Simulation, fctx: &mut FrameContext<'_>) {
        profiling::scope!("entity_render::render");
        self.cars.instances.clear();
        self.trucks.instances.clear();
        self.pedestrians.instances.clear();
        self.birds.instances.clear();
        for v in sim.world().vehicles.values() {
            let trans = &v.trans;
            let instance = MeshInstance {
                pos: trans.position,
                dir: trans.dir,
                tint: v.vehicle.tint.into(),
            };

            match v.vehicle.kind {
                VehicleKind::Car => self.cars.instances.push(instance),
                VehicleKind::Truck => self.trucks.instances.push(instance),
                _ => {}
            }
        }

        self.locomotives.instances.clear();
        self.wagons_passenger.instances.clear();
        self.wagons_freight.instances.clear();
        for wagon in sim.world().wagons.values() {
            let trans = &wagon.trans;
            let instance = MeshInstance {
                pos: trans.position,
                dir: trans.dir,
                tint: LinearColor::WHITE,
            };

            match wagon.wagon.kind {
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

        for p in sim.world().humans.values() {
            if matches!(p.location, Location::Outside) {
                self.pedestrians.instances.push(MeshInstance {
                    pos: p
                        .trans
                        .position
                        .up(0.5 + 0.4 * p.pedestrian.walk_anim.cos()),
                    dir: p.trans.dir.xy().z0(),
                    tint: LinearColor::WHITE,
                });
            }
        }

        for bird_ent in sim.world().birds.values() {
            self.birds.instances.push(MeshInstance {
                pos: bird_ent
                    .trans
                    .position
                    .up(0.5 + 0.4 * bird_ent.bird_mob.fly_anim.cos()),
                dir: bird_ent.trans.dir.xy().z0(),
                tint: LinearColor::WHITE,
            });
        }

        self.path_not_found.clear();
        for (_, (trans, itin)) in sim.world().query_trans_itin() {
            let Some(wait) = itin.is_wait_for_reroute() else {
                continue;
            };
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
        if let Some(x) = self.birds.build(fctx.gfx) {
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
