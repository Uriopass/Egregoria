use serde::{Deserialize, Serialize};

use egui_inspect::Inspect;
use geom::{Transform, Vec2};
use prototypes::{
    CompanyKind, GoodsCompanyID, GoodsCompanyPrototype, ItemID, Power, Recipe, DELTA,
};

use crate::economy::{find_trade_place, Market};
use crate::map::{Building, BuildingID, Map, Zone, MAX_ZONE_AREA};
use crate::map_dynamic::{BuildingInfos, ElectricityFlow};
use crate::souls::desire::WorkKind;
use crate::transportation::{spawn_parked_vehicle, VehicleKind};
use crate::utils::resources::Resources;
use crate::world::{CompanyEnt, HumanEnt, HumanID, VehicleID};
use crate::{ParCommandBuffer, SoulID, VehicleEnt};
use crate::{Simulation, World};

use super::desire::Work;

pub fn recipe_init(recipe: &Recipe, soul: SoulID, near: Vec2, market: &mut Market) {
    for item in &recipe.consumption {
        market.buy_until(soul, near, item.id, item.amount as u32)
    }
    for item in &recipe.production {
        market.register(soul, item.id);
    }
}

pub fn recipe_should_produce(recipe: &Recipe, soul: SoulID, market: &Market) -> bool {
    // Has enough resources
    recipe.consumption
            .iter()
            .all(move |item| market.capital(soul, item.id) >= item.amount)
            &&
            // Has enough storage
            recipe.production.iter().all(move |item| {
                market.capital(soul, item.id) < item.amount * (recipe.storage_multiplier + 1)
            })
        // has something to do
    && (!recipe.consumption.is_empty() || !recipe.production.is_empty())
}

pub fn recipe_act(recipe: &Recipe, soul: SoulID, near: Vec2, market: &mut Market) {
    for item in &recipe.consumption {
        market.produce(soul, item.id, -item.amount);
        market.buy_until(soul, near, item.id, item.amount as u32);
    }
    for item in &recipe.production {
        market.produce(soul, item.id, item.amount);
        market.sell_all(
            soul,
            near,
            item.id,
            (item.amount * recipe.storage_multiplier) as u32,
        );
    }
}

#[derive(Clone, Serialize, Deserialize, Inspect)]
pub struct GoodsCompanyState {
    pub proto: GoodsCompanyID,
    pub building: BuildingID,
    pub max_workers: u32,
    /// In [0; 1] range, to show how much has been made until new product
    pub progress: f32,
    pub driver: Option<HumanID>,
    pub trucks: Vec<VehicleID>,
}

impl CompanyEnt {
    /// Returns the productivity of the company, in [0; 1] range _before_ taking electricity into account
    pub fn raw_productivity(&self, proto: &GoodsCompanyPrototype, zone: Option<&Zone>) -> f32 {
        let mut p = 1.0;
        if proto.n_workers > 0 {
            p = self.workers.0.len() as f32 / proto.n_workers as f32;
        }
        if let Some(z) = zone {
            p *= z.area / MAX_ZONE_AREA
        }

        p
    }

    /// Returns the productivity of the company, in [0; 1] range
    pub fn productivity(
        &self,
        proto: &GoodsCompanyPrototype,
        zone: Option<&Zone>,
        map: &Map,
        elec_flow: &ElectricityFlow,
    ) -> f32 {
        let p = self.raw_productivity(proto, zone);

        if proto.power_consumption > Some(Power::ZERO) {
            if let Some(net_id) = map.electricity.net_id(self.comp.building) {
                if elec_flow.blackout(net_id) {
                    return 0.0;
                }
            }
        }

        p
    }
}

pub fn company_soul(
    sim: &mut Simulation,
    build_id: BuildingID,
    proto: GoodsCompanyID,
) -> Option<SoulID> {
    let proto = proto.prototype();

    let map = sim.map();
    let b = map.buildings().get(build_id)?;
    let door_pos = b.door_pos;
    let obb = b.obb;
    let height = b.height;
    drop(map);

    let ckind = proto.kind;
    let mut trucks = vec![];
    if ckind == CompanyKind::Factory {
        for _ in 0..proto.n_trucks {
            trucks.extend(spawn_parked_vehicle(sim, VehicleKind::Truck, door_pos))
        }
        if trucks.len() as u32 != proto.n_trucks {
            for truck in trucks {
                sim.write::<ParCommandBuffer<VehicleEnt>>().kill(truck);
            }
            return None;
        }
    }

    let comp = GoodsCompanyState {
        proto: proto.id,
        building: build_id,
        max_workers: proto.n_workers,
        progress: 0.0,
        driver: None,
        trucks,
    };

    let id = sim.world.insert(CompanyEnt {
        trans: Transform::new(obb.center().z(height)),
        comp,
        workers: Default::default(),
        sold: Default::default(),
        bought: Default::default(),
    });

    let company = &sim.world.get(id).unwrap().comp;

    let soul = SoulID::GoodsCompany(id);

    let job_opening = ItemID::new("job-opening");

    {
        let m = &mut *sim.write::<Market>();
        m.produce(soul, job_opening, company.max_workers as i32);
        m.sell_all(soul, door_pos.xy(), job_opening, 0);

        if let Some(ref r) = proto.recipe {
            recipe_init(r, soul, door_pos.xy(), m);
        }
    }

    sim.write::<BuildingInfos>()
        .set_owner(company.building, soul);

    Some(soul)
}

pub fn company_system(world: &mut World, res: &mut Resources) {
    profiling::scope!("souls::company_system");
    let cbuf: &ParCommandBuffer<CompanyEnt> = &res.read();
    let cbuf_human: &ParCommandBuffer<HumanEnt> = &res.read();
    let binfos: &BuildingInfos = &res.read();
    let market: &Market = &res.read();
    let map: &Map = &res.read();
    let elec_flow: &ElectricityFlow = &res.read();

    world.companies.iter_mut().for_each(|(me, c)| {
        let soul = SoulID::GoodsCompany(me);
        let b: &Building = unwrap_or!(map.buildings.get(c.comp.building), {
            cbuf.kill(me);
            return;
        });

        let proto = c.comp.proto.prototype();

        if let Some(recipe) = &proto.recipe {
            if recipe_should_produce(recipe, soul, market) {
                let productivity = c.productivity(proto, b.zone.as_ref(), map, elec_flow);

                c.comp.progress += productivity * DELTA / recipe.duration.seconds() as f32;
            }

            if c.comp.progress >= 1.0 {
                c.comp.progress -= 1.0;
                let kind = c.comp.proto;
                let bpos = b.door_pos;

                cbuf.exec_on(me, move |market| {
                    let recipe = kind.prototype().recipe.as_ref().unwrap();
                    recipe_act(recipe, soul, bpos.xy(), market);
                });
                return;
            }
        }

        for (_, trades) in c.bought.0.iter_mut() {
            for trade in trades.drain(..) {
                if let Some(owner_build) = find_trade_place(trade.seller, binfos) {
                    cbuf.exec_ent(me, move |sim| {
                        let (world, res) = sim.world_res();
                        if let Some(SoulID::FreightStation(owner)) =
                            res.read::<BuildingInfos>().owner(owner_build)
                        {
                            if let Some(f) = world.freight_stations.get_mut(owner) {
                                f.f.wanted_cargo += 1;
                            }
                        }
                    });
                }
            }
        }

        (|| {
            let Some(driver) = c.comp.driver else {
                return;
            };
            let Some(w) = world.humans.get(driver).and_then(|h| h.work.as_ref()) else {
                return;
            };
            if !matches!(
                w.kind,
                WorkKind::Driver {
                    deliver_order: None,
                    ..
                }
            ) {
                return;
            }
            let Some(trade) = c.sold.0.pop() else {
                return;
            };
            let Some(owner_build) = find_trade_place(trade.buyer, binfos) else {
                log::warn!("driver can't find the place to deliver for {:?}", &trade);
                return;
            };
            cbuf.exec_ent(me, move |sim| {
                let Some(h) = sim.world.humans.get_mut(driver) else {
                    return;
                };
                let Some(w) = h.work.as_mut() else {
                    return;
                };
                let WorkKind::Driver { deliver_order, .. } = &mut w.kind else {
                    return;
                };
                *deliver_order = Some(owner_build)
            });
        })();

        for &worker in c.workers.0.iter() {
            let Some(w) = world.humans.get(worker) else {
                continue;
            };

            if w.work.is_none() {
                let mut kind = WorkKind::Worker;

                if let Some(truck) = c.comp.trucks.first() {
                    if proto.kind == CompanyKind::Factory && c.comp.driver.is_none() {
                        kind = WorkKind::Driver {
                            deliver_order: None,
                            truck: *truck,
                        };

                        c.comp.driver = Some(worker);
                    }
                }

                let offset = common::rand::randu(common::hash_u64(worker) as u32);

                let b = c.comp.building;
                cbuf_human.exec_ent(worker, move |sim| {
                    let Some(w) = sim.world.humans.get_mut(worker) else {
                        return;
                    };
                    w.work = Some(Work::new(b, kind, offset));
                });
            }
        }
    });
}
