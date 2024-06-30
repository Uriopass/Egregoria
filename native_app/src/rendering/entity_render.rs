use common::FastMap;
use engine::{FrameContext, GfxContext, InstancedMeshBuilder, MeshInstance, SpriteBatchBuilder};
use geom::{LinearColor, Vec3, V3};
use prototypes::{RenderAsset, RollingStockID, RollingStockPrototype};
use simulation::transportation::{Location, VehicleKind};
use simulation::Simulation;

/// Render all entities using instanced rendering for performance
pub struct InstancedRender {
    pub path_not_found: SpriteBatchBuilder<true>,
    pub rolling_stock: FastMap<RollingStockID, InstancedMeshBuilder<true>>,
    pub cars: InstancedMeshBuilder<true>,
    // pub locomotives: InstancedMeshBuilder<true>,
    // pub wagons_passenger: InstancedMeshBuilder<true>,
    // pub wagons_freight: InstancedMeshBuilder<true>,
    pub trucks: InstancedMeshBuilder<true>,
    pub pedestrians: InstancedMeshBuilder<true>,
}

impl InstancedRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        defer!(log::info!("finished init of instanced render"));

        let mut rolling_stock = FastMap::default();
        RollingStockPrototype::iter()
            .map(|rail_wagon_proto| (&rail_wagon_proto.asset, rail_wagon_proto.id))
            .filter_map(|(asset, id)| {
                let RenderAsset::Mesh { path } = asset else {
                    None?
                };
                match gfx.mesh(path) {
                    Err(e) => {
                        log::error!("Failed to load mesh {}: {:?}", asset, e);
                        None
                    }
                    Ok(m) => Some((id, m)),
                }
            })
            .for_each(|(id, mesh)| {
                rolling_stock.insert(id, InstancedMeshBuilder::new_ref(&mesh));
            });

        let car = gfx.mesh("simple_car.glb".as_ref()).unwrap();
        InstancedRender {
            path_not_found: SpriteBatchBuilder::new(
                &gfx.texture("assets/sprites/path_not_found.png", "path_not_found"),
                gfx,
            ),

            rolling_stock,

            cars: InstancedMeshBuilder::new_ref(&car),
            // locomotives: InstancedMeshBuilder::new_ref(&gfx.mesh("train.glb".as_ref()).unwrap()),
            // wagons_freight: InstancedMeshBuilder::new_ref(&gfx.mesh("wagon_freight.glb".as_ref()).unwrap()),
            // wagons_passenger: InstancedMeshBuilder::new_ref(&gfx.mesh("wagon.glb".as_ref()).unwrap()),
            trucks: InstancedMeshBuilder::new_ref(&gfx.mesh("truck.glb".as_ref()).unwrap()),
            pedestrians: InstancedMeshBuilder::new_ref(
                &gfx.mesh("pedestrian.glb".as_ref()).unwrap(),
            ),
        }
    }

    pub fn render(&mut self, sim: &Simulation, fctx: &mut FrameContext<'_>) {
        profiling::scope!("entity_render::render");
        self.cars.instances.clear();
        self.trucks.instances.clear();
        self.pedestrians.instances.clear();
        for v in sim.world().vehicles.values() {
            let trans = &v.trans;
            let instance = MeshInstance {
                pos: trans.pos,
                dir: trans.dir,
                tint: v.vehicle.tint.into(),
            };

            match v.vehicle.kind {
                VehicleKind::Car => self.cars.instances.push(instance),
                VehicleKind::Truck => self.trucks.instances.push(instance),
                _ => {}
            }
        }

        self.rolling_stock.iter_mut().for_each(|(_, m)| {
            m.instances.clear();
        });
        for wagon in sim.world().wagons.values() {
            let trans = &wagon.trans;
            let instance = MeshInstance {
                pos: trans.pos,
                dir: trans.dir,
                tint: LinearColor::WHITE,
            };

            if let Some(mesh) = self.rolling_stock.get_mut(&wagon.wagon.rolling_stock) {
                mesh.instances.push(instance);
            }
        }

        for p in sim.world().humans.values() {
            if matches!(p.location, Location::Outside) {
                self.pedestrians.instances.push(MeshInstance {
                    pos: p.trans.pos.up(0.5 + 0.4 * p.pedestrian.walk_anim.cos()),
                    dir: p.trans.dir.xy().z0(),
                    tint: LinearColor::WHITE,
                });
            }
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
                trans.pos + off * 3.0 * V3::Y + 3.0 * V3::Z,
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

        self.rolling_stock.iter_mut().for_each(|(_, imb)| {
            if let Some(x) = imb.build(fctx.gfx) {
                fctx.objs.push(Box::new(x));
            }
        });
    }
}
