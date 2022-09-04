use super::desire::Work;
use crate::economy::{find_trade_place, ItemID, ItemRegistry, Market, Sold, Workers};
use crate::engine_interaction::Selectable;
use crate::map::{BuildingGen, BuildingID, Map};
use crate::map_dynamic::BuildingInfos;
use crate::souls::desire::WorkKind;
use crate::utils::time::GameTime;
use crate::vehicles::VehicleID;
use crate::{Egregoria, ParCommandBuffer, SoulID};
use common::saveload::Encoder;
use egui_inspect::Inspect;
use geom::{Transform, Vec2};
use hecs::{Entity, World};
use resources::Resources;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

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
                });

            log::info!("loaded {:?}", &self.descriptions[id]);
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
    pub driver: Option<SoulID>,
    pub trucks: Vec<VehicleID>,
}

pub fn company_soul(goria: &mut Egregoria, company: GoodsCompany) -> Option<SoulID> {
    let map = goria.map();
    let b = map.buildings().get(company.building)?;
    let door_pos = b.door_pos;
    let obb = b.obb;
    let height = b.height;
    drop(map);

    let e = goria.world.spawn(());

    let soul = SoulID(e);

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

    goria
        .world
        .insert(
            e,
            (
                company,
                Workers::default(),
                Sold::default(),
                Transform::new(obb.center().z(height)),
                Selectable::new(obb.axis()[0].mag() * 0.5),
            ),
        )
        .unwrap();

    Some(soul)
}

#[profiling::function]
pub fn company_system(world: &mut World, res: &mut Resources) {
    let ra = res.get().unwrap();
    let rb = res.get().unwrap();
    let rc = res.get().unwrap();
    let rd = res.get().unwrap();
    let re = res.get().unwrap();
    for (ent, (a, b, c)) in world
        .query::<(&mut GoodsCompany, &mut Sold, &Workers)>()
        .iter()
    {
        company(&*ra, &*rb, &*rc, &*rd, &*re, ent, a, b, c, world);
    }
}

pub fn company(
    time: &GameTime,
    cbuf: &ParCommandBuffer,
    binfos: &BuildingInfos,
    market: &Market,
    map: &Map,
    me: Entity,
    company: &mut GoodsCompany,
    sold: &mut Sold,
    workers: &Workers,
    world: &World,
) {
    let n_workers = workers.0.len();
    let soul = SoulID(me);
    let b = unwrap_or!(map.buildings.get(company.building), {
        cbuf.kill(me);
        return;
    });

    if company.recipe.should_produce(soul, market) {
        company.progress += n_workers as f32
            / (company.recipe.complexity as f32 * company.max_workers as f32)
            * time.delta;
    }

    if company.progress >= 1.0 {
        company.progress = 0.0;
        let recipe = company.recipe.clone();
        let bpos = b.door_pos;

        cbuf.exec_on(soul.0, move |market| {
            recipe.act(soul, bpos.xy(), market);
        });
        return;
    }

    if let Some(trade) = sold.0.drain(..1.min(sold.0.len())).next() {
        if let Some(driver) = company.driver {
            if let Ok(w) = world.get::<&Work>(driver.0) {
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
                        log::info!("asked driver to deliver");

                        cbuf.exec_ent(soul.0, move |goria| {
                            if let Some(mut w) = goria.comp_mut::<Work>(driver.0) {
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

    for &worker in workers.0.iter() {
        if world.get::<&Work>(worker.0).is_err() {
            let mut kind = WorkKind::Worker;

            if let Some(truck) = company.trucks.get(0) {
                if matches!(company.kind, CompanyKind::Factory { .. }) && company.driver.is_none() {
                    kind = WorkKind::Driver {
                        deliver_order: None,
                        truck: *truck,
                    };

                    company.driver = Some(worker);
                }
            }

            let offset = common::rand::randu(common::hash_u64(worker) as u32);

            cbuf.add_component(worker.0, Work::new(company.building, kind, offset))
        }
    }
}
