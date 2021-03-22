use super::desire::Desire;
use super::desire::Work;
use crate::economy::{CommodityKind, Market, Sold, Workers};
use crate::map_dynamic::BuildingInfos;
use crate::souls::desire::{DriverState, WorkKind};
use crate::vehicles::VehicleID;
use crate::{Egregoria, ParCommandBuffer, SoulID};
use common::GameTime;
use geom::Vec2;
use legion::world::SubWorld;
use legion::{system, Entity, EntityStore};
use map_model::{BuildingGen, BuildingID, BuildingKind, Map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
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

register_resource_noserialize!(GoodsCompanyRegistry);
pub struct GoodsCompanyRegistry {
    pub descriptions: HashMap<BuildingKind, GoodsCompanyDescription>,
}

impl Default for GoodsCompanyRegistry {
    fn default() -> Self {
        Self {
            descriptions: vec![
                GoodsCompanyDescription {
                    name: "Supermarket",
                    bkind: BuildingKind::Company(23),
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
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/supermarket.png",
                },
                GoodsCompanyDescription {
                    name: "Clothes store",
                    bkind: BuildingKind::Company(22),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Cloth, 1)], // TODO: actually implement stores
                        production: vec![(CommodityKind::Cloth, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 10.0,
                    asset_location: "assets/clothes_store.png",
                },
                GoodsCompanyDescription {
                    name: "Cloth factory",
                    bkind: BuildingKind::Company(21),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Polyester, 1), (CommodityKind::Wool, 1)],
                        production: vec![(CommodityKind::Cloth, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/cloth_factory.png",
                },
                GoodsCompanyDescription {
                    name: "Polyester refinery",
                    bkind: BuildingKind::Company(20),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Oil, 1)],
                        production: vec![(CommodityKind::Polyester, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 80.0,
                    asset_location: "assets/polyester_refinery.png",
                },
                GoodsCompanyDescription {
                    name: "Oil pump",
                    bkind: BuildingKind::Company(19),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Oil, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 20.0,
                    asset_location: "assets/oil_pump.png",
                },
                GoodsCompanyDescription {
                    name: "Textile processing facility",
                    bkind: BuildingKind::Company(18),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Wool, 1)],
                        production: vec![(CommodityKind::Cloth, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/textile_processing_facility.png",
                },
                GoodsCompanyDescription {
                    name: "Wool farm",
                    bkind: BuildingKind::Company(17),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Wool, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/wool_farm.png",
                },
                GoodsCompanyDescription {
                    name: "Florist",
                    bkind: BuildingKind::Company(16),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Flower, 1)], // TODO: actually implement stores
                        production: vec![(CommodityKind::Flower, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 10.0,
                    asset_location: "assets/florist.png",
                },
                GoodsCompanyDescription {
                    name: "Horticulturalist",
                    bkind: BuildingKind::Company(15),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Flower, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 80.0,
                    asset_location: "assets/horticulturalist.png",
                },
                GoodsCompanyDescription {
                    name: "High tech store",
                    bkind: BuildingKind::Company(14),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::HighTechProduct, 1)], // TODO: actually implement stores
                        production: vec![(CommodityKind::HighTechProduct, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/hightech_store.png",
                },
                GoodsCompanyDescription {
                    name: "High tech facility",
                    bkind: BuildingKind::Company(13),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::RareMetal, 1), (CommodityKind::Metal, 1)],
                        production: vec![(CommodityKind::HighTechProduct, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/hightech_facility.png",
                },
                GoodsCompanyDescription {
                    name: "Rare metal mine",
                    bkind: BuildingKind::Company(12),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::RareMetal, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/rare_metal_mine.png",
                },
                GoodsCompanyDescription {
                    name: "Furniture store",
                    bkind: BuildingKind::Company(11),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Metal, 1), (CommodityKind::WoodPlank, 1)],
                        production: vec![(CommodityKind::Furniture, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/furniture_store.png",
                },
                GoodsCompanyDescription {
                    name: "Foundry",
                    bkind: BuildingKind::Company(10),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::IronOre, 1)],
                        production: vec![(CommodityKind::Metal, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/foundry.png",
                },
                GoodsCompanyDescription {
                    name: "Iron mine",
                    bkind: BuildingKind::Company(9),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::IronOre, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/iron_mine.png",
                },
                GoodsCompanyDescription {
                    name: "Woodmill",
                    bkind: BuildingKind::Company(8),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::TreeLog, 1)],
                        production: vec![(CommodityKind::WoodPlank, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/woodmill.png",
                },
                GoodsCompanyDescription {
                    name: "Lumber yard",
                    bkind: BuildingKind::Company(7),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::TreeLog, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/lumber_yard.png",
                },
                GoodsCompanyDescription {
                    name: "Meat facility",
                    bkind: BuildingKind::Company(6),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 0.6,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::RawMeat, 1)],
                        production: vec![(CommodityKind::Meat, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/meat_facility.png",
                },
                GoodsCompanyDescription {
                    name: "Slaughterhouse",
                    bkind: BuildingKind::Company(5),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Carcass, 1)],
                        production: vec![(CommodityKind::RawMeat, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 50.0,
                    asset_location: "assets/slaughterhouse.png",
                },
                GoodsCompanyDescription {
                    name: "Animal Farm",
                    bkind: BuildingKind::Company(4),
                    bgen: BuildingGen::Farm,
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Cereal, 1)],
                        production: vec![(CommodityKind::Carcass, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 5,
                    size: 80.0,
                    asset_location: "assets/animal_farm.png",
                },
                GoodsCompanyDescription {
                    name: "Vegetable Farm",
                    bkind: BuildingKind::Company(3),
                    bgen: BuildingGen::Farm,
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Vegetable, 2)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 70.0,
                    asset_location: "assets/vegetable_farm.png",
                },
                GoodsCompanyDescription {
                    name: "Cereal Farm",
                    bkind: BuildingKind::Company(2),
                    bgen: BuildingGen::Farm,
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![],
                        production: vec![(CommodityKind::Cereal, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 120.0,
                    asset_location: "assets/cereal_farm.png",
                },
                GoodsCompanyDescription {
                    name: "Cereal Factory",
                    bkind: BuildingKind::Company(1),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 0.6,
                    },
                    kind: CompanyKind::Factory { n_trucks: 1 },
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Cereal, 1)],
                        production: vec![(CommodityKind::Flour, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 10,
                    size: 80.0,
                    asset_location: "assets/flour_factory.png",
                },
                GoodsCompanyDescription {
                    name: "Bakery",
                    bkind: BuildingKind::Company(0),
                    bgen: BuildingGen::CenteredDoor {
                        vertical_factor: 1.0,
                    },
                    kind: CompanyKind::Store,
                    recipe: Recipe {
                        consumption: vec![(CommodityKind::Flour, 1)],
                        production: vec![(CommodityKind::Bread, 1)],
                        complexity: 1000,
                        storage_multiplier: 5,
                    },
                    n_workers: 3,
                    size: 10.0,
                    asset_location: "assets/bakery.png",
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
    }

    pub fn should_produce(&self, soul: SoulID, market: &Market) -> bool {
        // Has enough ressources
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

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum CompanyKind {
    // Buyers come to get their goods
    Store,
    // Buyers get their goods delivered to them
    Factory { n_trucks: u32 },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GoodsCompany {
    pub kind: CompanyKind,
    pub recipe: Recipe,
    pub building: BuildingID,
    pub workers: i32,
    pub work_seconds: f32,
    pub driver: Option<SoulID>,
    pub trucks: Vec<VehicleID>,
}

impl GoodsCompany {
    pub fn progress(&self) -> f32 {
        self.work_seconds / (self.workers * self.recipe.complexity) as f32
    }
}

pub fn company_soul(goria: &mut Egregoria, company: GoodsCompany) -> SoulID {
    let bpos = goria.read::<Map>().buildings()[company.building].door_pos;

    let e = goria.world.push(());

    let soul = SoulID(e);

    {
        let m = &mut *goria.write::<Market>();
        m.produce(soul, CommodityKind::JobOpening, company.workers);
        m.sell_all(soul, bpos, CommodityKind::JobOpening);

        company.recipe.init(soul, bpos, m);
    }

    goria
        .write::<BuildingInfos>()
        .set_owner(company.building, soul);

    goria
        .world
        .push_with_id(e, (company, Workers::default(), Sold::default()));

    soul
}

register_system!(company);
#[system(par_for_each)]
#[read_component(Desire<Work>)]
pub fn company(
    #[resource] time: &GameTime,
    #[resource] cbuf: &ParCommandBuffer,
    #[resource] binfos: &BuildingInfos,
    #[resource] market: &Market,
    #[resource] map: &Map,
    me: &Entity,
    company: &mut GoodsCompany,
    sold: &mut Sold,
    workers: &Workers,
    sw: &SubWorld,
) {
    let n_workers = workers.0.len();
    let soul = SoulID(*me);

    if company.recipe.should_produce(soul, market) {
        company.work_seconds += n_workers as f32 * time.delta;
    }

    if company.work_seconds >= (company.recipe.complexity * company.workers) as f32 {
        company.work_seconds = 0.0;
        let recipe = company.recipe.clone();
        let bpos = map.buildings()[company.building].door_pos;

        cbuf.exec(move |goria| {
            recipe.act(soul, bpos, &mut *goria.write::<Market>());
        });
    }

    if let Some(driver) = company.driver {
        let driver_work_kind = sw
            .entry_ref(driver.0)
            .unwrap()
            .get_component::<Desire<Work>>()
            .unwrap()
            .v
            .kind;

        if let WorkKind::Driver {
            state: DriverState::WaitingForDelivery,
            ..
        } = driver_work_kind
        {
            if let Some(trade) = sold.0.drain(..1.min(sold.0.len())).next() {
                let owner_build = binfos.building_owned_by(trade.buyer).unwrap();

                log::info!("asked driver to deliver");

                cbuf.exec(move |goria| {
                    let w = goria.comp_mut::<Desire<Work>>(driver.0).unwrap();
                    if let WorkKind::Driver { ref mut state, .. } = w.v.kind {
                        *state = DriverState::Delivering(owner_build)
                    }
                })
            }
        }
    }

    for &worker in workers.0.iter() {
        if sw
            .entry_ref(worker.0)
            .unwrap()
            .get_component::<Desire<Work>>()
            .is_err()
        {
            let mut kind = WorkKind::Worker;
            if matches!(company.kind, CompanyKind::Factory { .. })
                && company.driver.is_none()
                && !company.trucks.is_empty()
            {
                kind = WorkKind::Driver {
                    state: DriverState::GoingToWork,
                    truck: company.trucks[0],
                };

                company.driver = Some(worker);
            }

            cbuf.add_component(
                worker.0,
                Desire::<Work>::new(Work::new(company.building, kind, 0.0)),
            )
        }
    }
}
