use super::desire::Desire;
use super::desire::Work;
use crate::economy::{JobApplication, Market, Sold, Wheat, Workers};
use crate::map_dynamic::BuildingInfos;
use crate::souls::desire::{DriverState, WorkKind};
use crate::vehicles::{spawn_parked_vehicle, VehicleID, VehicleKind};
use crate::{Egregoria, ParCommandBuffer, SoulID};
use common::GameTime;
use legion::world::SubWorld;
use legion::{system, Entity, EntityStore};
use map_model::{BuildingID, Map};

pub struct Farm {
    building: BuildingID,
    progress: f32,
    truck: VehicleID,
    driver: Option<SoulID>,
}

pub fn farm_soul(goria: &mut Egregoria, farm: BuildingID) -> Option<()> {
    let farmpos = goria.read::<Map>().buildings()[farm].door_pos;

    let truck = spawn_parked_vehicle(goria, VehicleKind::Truck, farmpos)?;

    let e = goria.world.push((
        Farm {
            building: farm,
            progress: 0.0,
            truck,
            driver: None,
        },
        Workers::default(),
    ));

    let soul = SoulID(e);

    let m: &mut Market<JobApplication> = &mut *goria.write::<Market<JobApplication>>();
    m.produce(soul, 10);
    m.sell(soul, farmpos, 10);
    goria.write::<BuildingInfos>().set_owner(farm, soul);

    Some(())
}

const SECONDS_PER_HARVEST: f32 = 1000.0;

#[system(par_for_each)]
#[read_component(Desire<Work>)]
pub fn farm(
    #[resource] time: &GameTime,
    #[resource] cbuf: &ParCommandBuffer,
    #[resource] binfos: &BuildingInfos,
    me: &Entity,
    farm: &mut Farm,
    sold: &mut Sold<Wheat>,
    workers: &Workers,
    sw: &SubWorld,
) {
    let n_workers = workers.0.len();

    farm.progress += n_workers as f32 * time.delta / SECONDS_PER_HARVEST;
    let soul = SoulID(*me);

    if farm.progress >= 1.0 {
        farm.progress = 0.0;
        let build = farm.building;

        cbuf.exec(move |goria| {
            let dpos = goria.read::<Map>().buildings()[build].door_pos;

            let mut market = goria.write::<Market<Wheat>>();
            market.produce(soul, 1);
            let cap = market.capital(soul);
            market.sell(soul, dpos, cap);
        })
    }

    if let Some(driver) = farm.driver {
        let kind = sw
            .entry_ref(driver.0)
            .unwrap()
            .get_component::<Desire<Work>>()
            .unwrap()
            .v
            .kind;

        if let WorkKind::Driver { state, .. } = kind {
            if let DriverState::WaitingForDelivery = state {
                if let Some(trade) = sold.0.drain(..1).next() {
                    if let Some(owner_build) = binfos.building_owned_by(trade.buyer) {
                        cbuf.exec(move |goria| {
                            let w = goria.comp_mut::<Desire<Work>>(driver.0).unwrap();
                            if let WorkKind::Driver { ref mut state, .. } = w.v.kind {
                                *state = DriverState::Delivering(owner_build)
                            }
                        })
                    }
                }
            }
        }
    }

    for worker in workers.0.iter() {
        if sw
            .entry_ref(worker.buyer.0)
            .unwrap()
            .get_component::<Desire<Work>>()
            .is_err()
        {
            let mut kind = WorkKind::Worker;
            if farm.driver.is_none() {
                kind = WorkKind::Driver {
                    state: DriverState::GoingToWork,
                    truck: farm.truck,
                };
                farm.driver = Some(worker.buyer);
            }

            cbuf.add_component(
                worker.buyer.0,
                Desire::<Work>::new(Work::new(farm.building, kind, 0.0)),
            )
        }
    }
}
