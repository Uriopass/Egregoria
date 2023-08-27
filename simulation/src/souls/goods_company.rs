use super::desire::Work;
use crate::economy::{find_trade_place, ItemID, ItemRegistry, Market};
use crate::map::{Building, BuildingID, Map, Zone, MAX_ZONE_AREA};
use crate::map_dynamic::BuildingInfos;
use crate::souls::desire::WorkKind;
use crate::utils::resources::Resources;
use crate::utils::time::GameTime;
use crate::world::{CompanyEnt, HumanEnt, HumanID, VehicleID};
use crate::{ParCommandBuffer, SoulID};
use crate::{Simulation, World};
use common::descriptions::{
    BuildingGen, CompanyKind, GoodsCompanyDescriptionJSON, ZoneDescription,
};
use common::saveload::Encoder;
use egui_inspect::Inspect;
use geom::{Transform, Vec2};
use serde::{Deserialize, Serialize};
use slotmapd::{new_key_type, SlotMap};

#[derive(Debug, Clone, Serialize, Deserialize, Inspect)]
pub struct Recipe {
    pub consumption: Vec<(ItemID, i32)>,
    pub production: Vec<(ItemID, i32)>,

    /// Time to execute the recipe when the facility is at full capacity, in seconds
    pub complexity: i32,

    /// Quantity to store per production in terms of quantity produced. So if it takes 1ton of flour to make
    /// 1 ton of bread. A storage multiplier of 3 means 3 tons of bread will be stored before stopping to
    /// produce it.
    pub storage_multiplier: i32,
}

new_key_type! {
    pub struct GoodsCompanyID;
}

debug_inspect_impl!(GoodsCompanyID);

#[derive(Debug)]
pub struct GoodsCompanyDescription {
    pub id: GoodsCompanyID,
    pub name: String,
    pub bgen: BuildingGen,
    pub kind: CompanyKind,
    pub recipe: Recipe,
    pub n_workers: i32,
    pub size: f32,
    pub asset_location: String,
    pub price: i64,
    pub zone: Option<Box<ZoneDescription>>,
}

#[derive(Default)]
pub struct GoodsCompanyRegistry {
    pub descriptions: SlotMap<GoodsCompanyID, GoodsCompanyDescription>,
}

impl GoodsCompanyRegistry {
    pub fn load(&mut self, source: &str, registry: &ItemRegistry) {
        let descriptions: Vec<GoodsCompanyDescriptionJSON> =
            match common::saveload::JSON::decode(source.as_ref()) {
                Ok(x) => x,
                Err(e) => {
                    log::error!("couldn't load goods company descriptions: {}", e);
                    return;
                }
            };

        for descr in descriptions {
            let recipe = Recipe {
                consumption: descr
                    .recipe
                    .consumption
                    .into_iter()
                    .map(|(item, qty)| {
                        let item_id = registry.id(&item);
                        (item_id, qty)
                    })
                    .collect(),
                production: descr
                    .recipe
                    .production
                    .into_iter()
                    .map(|(item, qty)| {
                        let item_id = registry.id(&item);
                        (item_id, qty)
                    })
                    .collect(),
                complexity: descr.recipe.complexity,
                storage_multiplier: descr.recipe.storage_multiplier,
            };

            #[allow(unused_variables)]
            let id = self
                .descriptions
                .insert_with_key(move |id| GoodsCompanyDescription {
                    id,
                    name: descr.name,
                    bgen: descr.bgen,
                    kind: descr.kind,
                    recipe,
                    n_workers: descr.n_workers,
                    size: descr.size,
                    asset_location: descr.asset_location,
                    price: descr.price,
                    zone: descr.zone,
                });

            #[cfg(not(test))]
            log::debug!("loaded {:?}", &self.descriptions[id]);
        }
    }
}

impl Recipe {
    pub fn init(&self, soul: SoulID, near: Vec2, market: &mut Market) {
        for &(kind, qty) in &self.consumption {
            market.buy_until(soul, near, kind, qty as u32)
        }
        for &(kind, _) in &self.production {
            market.register(soul, kind);
        }
    }

    pub fn should_produce(&self, soul: SoulID, market: &Market) -> bool {
        // Has enough resources
        self.consumption
            .iter()
            .all(move |&(kind, qty)| market.capital(soul, kind) >= qty)
            &&
            // Has enough storage
            self.production.iter().all(move |&(kind, qty)| {
                market.capital(soul, kind) < qty * (self.storage_multiplier + 1)
            })
    }

    pub fn act(&self, soul: SoulID, near: Vec2, market: &mut Market) {
        for &(kind, qty) in &self.consumption {
            market.produce(soul, kind, -qty);
            market.buy_until(soul, near, kind, qty as u32);
        }
        for &(kind, qty) in &self.production {
            market.produce(soul, kind, qty);
            market.sell_all(soul, near, kind, (qty * self.storage_multiplier) as u32);
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Inspect)]
pub struct GoodsCompany {
    pub kind: CompanyKind,
    pub recipe: Recipe,
    pub building: BuildingID,
    pub max_workers: i32,
    /// In [0; 1] range, to show how much has been made until new product
    pub progress: f32,
    pub driver: Option<HumanID>,
    pub trucks: Vec<VehicleID>,
}

impl GoodsCompany {
    pub fn productivity(&self, workers: usize, zone: Option<&Zone>) -> f32 {
        workers as f32 / self.max_workers as f32 * zone.map_or(1.0, |z| z.area / MAX_ZONE_AREA)
    }
}

pub fn company_soul(sim: &mut Simulation, company: GoodsCompany) -> Option<SoulID> {
    let map = sim.map();
    let b = map.buildings().get(company.building)?;
    let door_pos = b.door_pos;
    let obb = b.obb;
    let height = b.height;
    drop(map);

    let id = sim.world.insert(CompanyEnt {
        trans: Transform::new(obb.center().z(height)),
        comp: company,
        workers: Default::default(),
        sold: Default::default(),
        bought: Default::default(),
    });

    let company = &sim.world.get(id).unwrap().comp;

    let soul = SoulID::GoodsCompany(id);

    let job_opening = sim.read::<ItemRegistry>().id("job-opening");

    {
        let m = &mut *sim.write::<Market>();
        m.produce(soul, job_opening, company.max_workers);
        m.sell_all(soul, door_pos.xy(), job_opening, 0);

        company.recipe.init(soul, door_pos.xy(), m);
    }

    sim.write::<BuildingInfos>()
        .set_owner(company.building, soul);

    Some(soul)
}

pub fn company_system(world: &mut World, res: &mut Resources) {
    profiling::scope!("souls::company_system");
    let delta = res.read::<GameTime>().realdelta;
    let cbuf: &ParCommandBuffer<CompanyEnt> = &res.read();
    let cbuf_human: &ParCommandBuffer<HumanEnt> = &res.read();
    let binfos: &BuildingInfos = &res.read();
    let market: &Market = &res.read();
    let map: &Map = &res.read();

    world.companies.iter_mut().for_each(|(me, c)| {
        let n_workers = c.workers.0.len();
        let soul = SoulID::GoodsCompany(me);
        let b: &Building = unwrap_or!(map.buildings.get(c.comp.building), {
            cbuf.kill(me);
            return;
        });

        if c.comp.recipe.should_produce(soul, market) {
            c.comp.progress += c.comp.productivity(n_workers, b.zone.as_ref())
                / c.comp.recipe.complexity as f32
                * delta;
        }

        if c.comp.progress >= 1.0 {
            c.comp.progress -= 1.0;
            let recipe = c.comp.recipe.clone();
            let bpos = b.door_pos;

            cbuf.exec_on(me, move |market| {
                recipe.act(soul, bpos.xy(), market);
            });
            return;
        }

        for (_, trades) in c.bought.0.iter_mut() {
            for trade in trades.drain(..) {
                if let Some(owner_build) =
                    find_trade_place(trade.seller, b.door_pos.xy(), binfos, map)
                {
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
            let Some(trade) = c.sold.0.pop() else {
                return;
            };
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
            let Some(owner_build) = find_trade_place(trade.buyer, b.door_pos.xy(), binfos, map)
            else {
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

                if let Some(truck) = c.comp.trucks.get(0) {
                    if matches!(c.comp.kind, CompanyKind::Factory { .. }) && c.comp.driver.is_none()
                    {
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
