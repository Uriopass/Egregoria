use super::desire::Work;
use crate::economy::{find_trade_place, ItemID, ItemRegistry, Market};
use crate::map::{Building, BuildingGen, BuildingID, Map, Zone, MAX_ZONE_AREA};
use crate::map_dynamic::BuildingInfos;
use crate::souls::desire::WorkKind;
use crate::utils::resources::Resources;
use crate::utils::time::GameTime;
use crate::world::{CompanyEnt, HumanEnt, HumanID, VehicleID};
use crate::World;
use crate::{Egregoria, ParCommandBuffer, SoulID};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneDescription {
    pub floor: String,
    pub filler: String,
    /// The price for each "production unit"
    pub price_per_area: i64,
    /// Wether the zone filler positions should be randomized
    #[serde(default)]
    pub randomize_filler: bool,
}

#[derive(Default)]
pub struct GoodsCompanyRegistry {
    pub descriptions: SlotMap<GoodsCompanyID, GoodsCompanyDescription>,
}

#[derive(Serialize, Deserialize)]
struct RecipeDescription {
    pub consumption: Vec<(String, i32)>,
    pub production: Vec<(String, i32)>,
    pub complexity: i32,
    pub storage_multiplier: i32,
}

#[derive(Serialize, Deserialize)]
struct BuildingGenDescription {
    pub kind: String,
    pub vertical_factor: Option<f32>,
    pub door_pos: Option<Vec2>,
}

#[derive(Serialize, Deserialize)]
struct GoodsCompanyDescriptionJSON {
    pub name: String,
    pub bgen: BuildingGenDescription,
    pub kind: String,
    pub recipe: RecipeDescription,
    pub n_workers: i32,
    pub n_trucks: Option<u32>,
    pub size: f32,
    pub asset_location: String,
    pub price: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<Box<ZoneDescription>>,
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
            let kind = match descr.kind.as_ref() {
                "store" => CompanyKind::Store,
                "network" => CompanyKind::Network,
                "factory" => CompanyKind::Factory {
                    n_trucks: descr
                        .n_trucks
                        .expect("expecting n_trucks when using kind factory"),
                },
                _ => {
                    log::error!("unknown goods company kind: {}", descr.kind);
                    continue;
                }
            };
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

            let bgen = match descr.bgen.kind.as_ref() {
                "farm" => BuildingGen::Farm,
                "centered_door" => BuildingGen::CenteredDoor {
                    vertical_factor: descr
                        .bgen
                        .vertical_factor
                        .expect("expecting vertical factor when using centered_door"),
                },
                "no_walkway" => BuildingGen::NoWalkway {
                    door_pos: descr
                        .bgen
                        .door_pos
                        .expect("expecting door_pos when using no_walkway"),
                },
                _ => {
                    log::error!("unknown building gen kind: {}", descr.bgen.kind);
                    continue;
                }
            };
            #[allow(unused_variables)]
            let id = self
                .descriptions
                .insert_with_key(move |id| GoodsCompanyDescription {
                    id,
                    name: descr.name,
                    bgen,
                    kind,
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

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum CompanyKind {
    // Buyers come to get their goods
    Store,
    // Buyers get their goods delivered to them
    Factory { n_trucks: u32 },
    // Buyers get their goods instantly delivered, useful for things like electricity/water/..
    Network,
}

debug_inspect_impl!(CompanyKind);

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

pub fn company_soul(goria: &mut Egregoria, company: GoodsCompany) -> Option<SoulID> {
    let map = goria.map();
    let b = map.buildings().get(company.building)?;
    let door_pos = b.door_pos;
    let obb = b.obb;
    let height = b.height;
    drop(map);

    let id = goria.world.insert(CompanyEnt {
        trans: Transform::new(obb.center().z(height)),
        comp: company,
        workers: Default::default(),
        sold: Default::default(),
        bought: Default::default(),
    });

    let company = &goria.world.get(id).unwrap().comp;

    let soul = SoulID::GoodsCompany(id);

    let job_opening = goria.read::<ItemRegistry>().id("job-opening");

    {
        let m = &mut *goria.write::<Market>();
        m.produce(soul, job_opening, company.max_workers);
        m.sell_all(soul, door_pos.xy(), job_opening, 0);

        company.recipe.init(soul, door_pos.xy(), m);
    }

    goria
        .write::<BuildingInfos>()
        .set_owner(company.building, soul);

    Some(soul)
}

#[profiling::function]
pub fn company_system(world: &mut World, res: &mut Resources) {
    let delta = res.get::<GameTime>().realdelta;
    let cbuf: &ParCommandBuffer<CompanyEnt> = &res.get();
    let cbuf_human: &ParCommandBuffer<HumanEnt> = &res.get();
    let binfos: &BuildingInfos = &res.get();
    let market: &Market = &res.get();
    let map: &Map = &res.get();

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
                    cbuf.exec_ent(me, move |goria| {
                        let (world, res) = goria.world_res();
                        if let Some(SoulID::FreightStation(owner)) =
                            res.get::<BuildingInfos>().owner(owner_build)
                        {
                            if let Some(mut f) = world.freight_stations.get_mut(owner) {
                                f.f.wanted_cargo += 1;
                            }
                        }
                    });
                }
            }
        }

        if let Some(trade) = c.sold.0.drain(..1.min(c.sold.0.len())).next() {
            if let Some(driver) = c.comp.driver {
                if let Some(ref mut w) = world.humans.get(driver).and_then(|h| h.work) {
                    if matches!(
                        w.kind,
                        WorkKind::Driver {
                            deliver_order: None,
                            ..
                        }
                    ) {
                        if let Some(owner_build) =
                            find_trade_place(trade.buyer, b.door_pos.xy(), binfos, map)
                        {
                            cbuf.exec_ent(me, move |goria| {
                                if let Some(ref mut w) =
                                    goria.world.humans.get(driver).and_then(|h| h.work)
                                {
                                    if let WorkKind::Driver {
                                        ref mut deliver_order,
                                        ..
                                    } = w.kind
                                    {
                                        *deliver_order = Some(owner_build)
                                    }
                                }
                            })
                        } else {
                            log::warn!("driver can't find the place to deliver for {:?}", &trade);
                        }
                    }
                }
            }
        }

        for &worker in c.workers.0.iter() {
            let Some(w) = world.humans.get(worker) else { continue; };

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
                cbuf_human.exec_ent(worker, move |goria| {
                    let Some(w) = goria.world.humans.get_mut(worker) else { return };
                    w.work = Some(Work::new(b, kind, offset));
                });
            }
        }
    });
}
