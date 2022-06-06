use super::desire::Work;
use crate::economy::{CommodityKind, Market, Sold, Workers};
use crate::engine_interaction::Selectable;
use crate::map_dynamic::BuildingInfos;
use crate::souls::desire::WorkKind;
use crate::utils::time::GameTime;
use crate::vehicles::VehicleID;
use crate::{Egregoria, ParCommandBuffer, SoulID};
use geom::{Transform, Vec2};
use hecs::{Entity, World};
use if_chain::if_chain;
use imgui_inspect_derive::Inspect;
use map_model::{BuildingGen, BuildingID, BuildingKind, Map};
use resources::Resources;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Serialize, Deserialize, Inspect)]
pub struct Recipe {
    pub consumption: Vec<(CommodityKind, i32)>,
    pub production: Vec<(CommodityKind, i32)>,

    /// Time to execute the recipe when the facility is at full capacity, in seconds
    pub complexity: i32,

    /// Quantity to store per production in terms of quantity produced. So if it takes 1ton of flour to make
    /// 1 ton of bread. A storage multiplier of 3 means 3 tons of bread will be stored before stopping to
    /// produce it.
    pub storage_multiplier: i32,
}

pub struct GoodsCompanyDescription {
    pub name: &'static str,
    pub bkind: BuildingKind,
    pub bgen: BuildingGen,
    pub kind: CompanyKind,
    pub recipe: Recipe,
    pub n_workers: i32,
    pub size: f32,
    pub asset_location: &'static str,
}

pub struct GoodsCompanyRegistry {
    pub descriptions: BTreeMap<BuildingKind, GoodsCompanyDescription>,
}

impl Default for GoodsCompanyRegistry {
    fn default() -> Self {
        Self {
            descriptions: vec![
                GoodsCompanyDescription {
                    name: "Coal power plant", // doesn't do anything yet, was really to test my modeling skills..
                    bkind: BuildingKind::GoodsCompany(25), // should've probably done an existing useful building instead..
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Network,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Coal, 1)],
                        production: vec![(CommodityKind::Electricity, 2460)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 165.0,
                    asset_location: "coal_power_plant.glb",
                },
                GoodsCompanyDescription {
                    name: "Useless warehouse",
                    bkind: BuildingKind::GoodsCompany(24),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![],
                        complexity: 1000,
                        storage_multiplier: 0,
                    },
                    n_workers: 100,
                    size: 100.0,
                    asset_location: "assets/warehouse.png",
                },
                GoodsCompanyDescription {
                    name: "Supermarket",
                    bkind: BuildingKind::GoodsCompany(23),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![
                            (CommodityKind::Meat, 1),
                            (CommodityKind::Vegetable, 1),
                            (CommodityKind::Cereal, 1),
                        ], // TODO: actually implement stores
                        production: vec![
                            (CommodityKind::Meat, 1),
                            (CommodityKind::Vegetable, 1),
                            (CommodityKind::Cereal, 1),
                        ],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/supermarket.png",
                },
                GoodsCompanyDescription {
                    name: "Clothes store",
                    bkind: BuildingKind::GoodsCompany(22),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Cloth, 1)], // TODO: actually implement stores
                        production: vec![(CommodityKind::Cloth, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 10.0,
                    asset_location: "assets/clothes_store.png",
                },
                GoodsCompanyDescription {
                    name: "Cloth factory",
                    bkind: BuildingKind::GoodsCompany(21),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Polyester, 1), (CommodityKind::Wool, 1)],
                        production: vec![(CommodityKind::Cloth, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/cloth_factory.png",
                },
                GoodsCompanyDescription {
                    name: "Polyester refinery",
                    bkind: BuildingKind::GoodsCompany(20),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Oil, 1)],
                        production: vec![(CommodityKind::Polyester, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 80.0,
                    asset_location: "assets/polyester_refinery.png",
                },
                GoodsCompanyDescription {
                    name: "Oil pump",
                    bkind: BuildingKind::GoodsCompany(19),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Oil, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 20.0,
                    asset_location: "assets/oil_pump.png",
                },
                GoodsCompanyDescription {
                    name: "Textile processing facility",
                    bkind: BuildingKind::GoodsCompany(18),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Wool, 1)],
                        production: vec![(CommodityKind::Cloth, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/textile_processing_facility.png",
                },
                GoodsCompanyDescription {
                    name: "Wool farm",
                    bkind: BuildingKind::GoodsCompany(17),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Wool, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/wool_farm.png",
                },
                GoodsCompanyDescription {
                    name: "Florist",
                    bkind: BuildingKind::GoodsCompany(16),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Flower, 1)], // TODO: actually implement stores
                        production: vec![(CommodityKind::Flower, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 10.0,
                    asset_location: "assets/florist.png",
                },
                GoodsCompanyDescription {
                    name: "Horticulturalist",
                    bkind: BuildingKind::GoodsCompany(15),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Flower, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 80.0,
                    asset_location: "assets/horticulturalist.png",
                },
                GoodsCompanyDescription {
                    name: "High tech store",
                    bkind: BuildingKind::GoodsCompany(14),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::HighTechProduct, 1)], // TODO: actually implement stores
                        production: vec![(CommodityKind::HighTechProduct, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/hightech_store.png",
                },
                GoodsCompanyDescription {
                    name: "High tech facility",
                    bkind: BuildingKind::GoodsCompany(13),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::RareMetal, 1), (CommodityKind::Metal, 1)],
                        production: vec![(CommodityKind::HighTechProduct, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/hightech_facility.png",
                },
                GoodsCompanyDescription {
                    name: "Rare metal mine",
                    bkind: BuildingKind::GoodsCompany(12),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::RareMetal, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/rare_metal_mine.png",
                },
                GoodsCompanyDescription {
                    name: "Furniture store",
                    bkind: BuildingKind::GoodsCompany(11),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Metal, 1), (CommodityKind::WoodPlank, 1)],
                        production: vec![(CommodityKind::Furniture, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/furniture_store.png",
                },
                GoodsCompanyDescription {
                    name: "Foundry",
                    bkind: BuildingKind::GoodsCompany(10),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::IronOre, 1)],
                        production: vec![(CommodityKind::Metal, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/foundry.png",
                },
                GoodsCompanyDescription {
                    name: "Iron mine",
                    bkind: BuildingKind::GoodsCompany(9),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::IronOre, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/iron_mine.png",
                },
                GoodsCompanyDescription {
                    name: "Woodmill",
                    bkind: BuildingKind::GoodsCompany(8),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::TreeLog, 1)],
                        production: vec![(CommodityKind::WoodPlank, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/woodmill.png",
                },
                GoodsCompanyDescription {
                    name: "Lumber yard",
                    bkind: BuildingKind::GoodsCompany(7),
                    bgen: BuildingGen::Farm,
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::TreeLog, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 200.0,
                    asset_location: "assets/lumber_yard.png",
                },
                GoodsCompanyDescription {
                    name: "Meat facility",
                    bkind: BuildingKind::GoodsCompany(6),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 0.6,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::RawMeat, 1)],
                        production: vec![(CommodityKind::Meat, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/meat_facility.png",
                },
                GoodsCompanyDescription {
                    name: "Slaughterhouse",
                    bkind: BuildingKind::GoodsCompany(5),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Carcass, 1)],
                        production: vec![(CommodityKind::RawMeat, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 50.0,
                    asset_location: "assets/slaughterhouse.png",
                },
                GoodsCompanyDescription {
                    name: "Animal Farm",
                    bkind: BuildingKind::GoodsCompany(4),
                    bgen: BuildingGen::Farm,
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Cereal, 1)],
                        production: vec![(CommodityKind::Carcass, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 80.0,
                    asset_location: "assets/animal_farm.png",
                },
                GoodsCompanyDescription {
                    name: "Vegetable Farm",
                    bkind: BuildingKind::GoodsCompany(3),
                    bgen: BuildingGen::Farm,
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Vegetable, 2)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 70.0,
                    asset_location: "assets/vegetable_farm.png",
                },
                GoodsCompanyDescription {
                    name: "Bakery",
                    bkind: BuildingKind::GoodsCompany(2),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Flour, 1)],
                        production: vec![(CommodityKind::Bread, 1)],
                        complexity: 100,
                        storage_multiplier: 5,
                    },
                    n_workers: 3,
                    size: 10.0,
                    asset_location: "bakery.glb",
                },
                GoodsCompanyDescription {
                    name: "Cereal Factory",
                    bkind: BuildingKind::GoodsCompany(1),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 0.6,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Cereal, 1)],
                        production: vec![(CommodityKind::Flour, 10)],
                        complexity: 200,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "flour_factory.glb",
                },
                GoodsCompanyDescription {
                    name: "Cereal Farm",
                    bkind: BuildingKind::GoodsCompany(0),
                    bgen: BuildingGen::Farm,
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Cereal, 1)],
                        complexity: 200,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 120.0,
                    asset_location: "assets/cereal_farm.png",
                },
            ]
            .into_iter()
            .map(|x| (x.bkind, x))
            .collect(),
        }
    }
}

impl Recipe {
    pub fn init(&self, soul: SoulID, near: Vec2, market: &mut Market) {
        for &(kind, qty) in &self.consumption {
            market.buy_until(soul, near, kind, qty)
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
                market.capital(soul, kind) < qty * self.storage_multiplier
            })
    }

    pub fn act(&self, soul: SoulID, near: Vec2, market: &mut Market) {
        for &(kind, qty) in &self.consumption {
            market.produce(soul, kind, -qty);
            market.buy_until(soul, near, kind, qty);
        }
        for &(kind, qty) in &self.production {
            market.produce(soul, kind, qty);
            market.sell_all(soul, near, kind);
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
    let b = &map.buildings().get(company.building)?;
    let door_pos = b.door_pos;
    let obb = b.obb;
    let height = b.height;
    drop(map);

    let e = goria.world.spawn(());

    let soul = SoulID(e);

    {
        let m = &mut *goria.write::<Market>();
        m.produce(soul, CommodityKind::JobOpening, company.max_workers);
        m.sell_all(soul, door_pos.xy(), CommodityKind::JobOpening);

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
                Selectable::new(obb.axis()[0].magnitude() * 0.5),
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

    if company.recipe.should_produce(soul, market) {
        company.progress += n_workers as f32
            / (company.recipe.complexity as f32 * company.max_workers as f32)
            * time.delta;
    }

    if company.progress >= 1.0 {
        company.progress = 0.0;
        let recipe = company.recipe.clone();
        let bpos = unwrap_or!(map.buildings().get(company.building), {
            cbuf.kill(me);
            return;
        })
        .door_pos;

        cbuf.exec_on(soul.0, move |market| {
            recipe.act(soul, bpos.xy(), market);
        });
        return;
    }

    if_chain! {
        if let Some(trade) = sold.0.drain(..1.min(sold.0.len())).next();
        if let Some(driver) = company.driver;
        if let Ok(w) = world.get::<Work>(driver.0);
        if matches!(w.kind, WorkKind::Driver { deliver_order: None, .. });
        if let Some(owner_build) = binfos.building_owned_by(trade.buyer);
        then {
            log::info!("asked driver to deliver");

            cbuf.exec_ent(soul.0, move |goria| {
                if let Some(mut w) = goria.comp_mut::<Work>(driver.0) {
                    if let WorkKind::Driver { ref mut deliver_order, .. } = w.kind {
                        *deliver_order = Some(owner_build)
                    }
                }
            })
        }
    }

    for &worker in workers.0.iter() {
        if world.get::<Work>(worker.0).is_err() {
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
