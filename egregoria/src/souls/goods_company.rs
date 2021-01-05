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
use map_model::{BuildingID, Map};

#[derive(Copy, Clone)]
pub struct Recipe {
    pub consumption: &'static [(CommodityKind, i32)],
    pub production: &'static [(CommodityKind, i32)],
    pub seconds_per_work: i32,

    /// Quantity to store per production in terms of quantity produced. So if it takes 1ton of flour to make
    /// 1 ton of bread. A storage multiplier of 3 means 3 tons of bread will be stored before stopping to
    /// produce it.
    pub storage_multiplier: i32,
}

impl Recipe {
    pub fn init(&self, soul: SoulID, near: Vec2, market: &mut Market) {
        for &(kind, qty) in self.consumption {
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
        for &(kind, qty) in self.consumption {
            market.produce(soul, kind, -qty);
            market.buy_until(soul, near, kind, qty);
        }
        for &(kind, qty) in self.production {
            market.produce(soul, kind, qty);
            market.sell_all(soul, near, kind);
        }
    }
}

#[derive(Copy, Clone)]
pub enum CompanyKind {
    // Buyers come to get their goods
    Store,
    // Buyers get their goods delivered to them
    Factory {
        truck: VehicleID,
        driver: Option<SoulID>,
    },
}

#[derive(Copy, Clone)]
pub struct GoodsCompany {
    pub kind: CompanyKind,
    pub recipe: Recipe,
    pub building: BuildingID,
    pub workers: i32,
    pub progress: f32,
}

pub fn company_soul(goria: &mut Egregoria, company: GoodsCompany) {
    let bpos = goria.read::<Map>().buildings()[company.building].door_pos;

    let e = goria
        .world
        .push((company, Workers::default(), Sold::default()));

    let soul = SoulID(e);

    let m: &mut Market = &mut *goria.write::<Market>();
    m.produce(soul, CommodityKind::JobOpening, company.workers);
    m.sell_all(soul, bpos, CommodityKind::JobOpening);

    company.recipe.init(soul, bpos, m);

    goria
        .write::<BuildingInfos>()
        .set_owner(company.building, soul);
}

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
        company.progress += n_workers as f32 * time.delta / company.recipe.seconds_per_work as f32;
    }

    if company.progress >= 1.0 {
        company.progress = 0.0;
        let recipe = company.recipe;
        let bpos = map.buildings()[company.building].door_pos;

        cbuf.exec(move |goria| {
            recipe.act(soul, bpos, &mut *goria.write::<Market>());
        });
    }

    if let CompanyKind::Factory {
        driver: Some(driver),
        ..
    } = company.kind
    {
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
            if let CompanyKind::Factory {
                truck,
                ref mut driver,
            } = company.kind
            {
                if driver.is_none() {
                    kind = WorkKind::Driver {
                        state: DriverState::GoingToWork,
                        truck,
                    };

                    *driver = Some(worker);
                }
            }

            cbuf.add_component(
                worker.0,
                Desire::<Work>::new(Work::new(company.building, kind, 0.0)),
            )
        }
    }
}
