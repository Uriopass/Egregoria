use super::desire::Desire;
use super::desire::Work;
use crate::economy::{JobApplication, Market, Wheat, Workers};
use crate::map_dynamic::BuildingInfos;
use crate::{Egregoria, ParCommandBuffer, SoulID};
use common::GameTime;
use legion::world::SubWorld;
use legion::{system, Entity, EntityStore};
use map_model::{BuildingID, Map};

pub struct Farm {
    building: BuildingID,
    progress: f32,
}

pub fn farm_soul(goria: &mut Egregoria, farm: BuildingID) {
    let farmpos = goria.read::<Map>().buildings()[farm].door_pos;

    let e = goria.world.push((
        Farm {
            building: farm,
            progress: 0.0,
        },
        Workers(vec![]),
    ));

    let soul = SoulID(e);

    let m: &mut Market<JobApplication> = &mut *goria.write::<Market<JobApplication>>();
    m.produce(soul, 10);
    m.sell(soul, farmpos, 10);
    goria.write::<BuildingInfos>().set_owner(farm, soul);
}

const SECONDS_PER_HARVEST: f32 = 1000.0;

#[system(for_each)]
#[read_component(Desire<Work>)]
pub fn farm_assign_workers(
    #[resource] time: &GameTime,
    #[resource] cbuf: &ParCommandBuffer,
    farm: &Farm,
    workers: &Workers,
    sw: &SubWorld,
) {
    if !time.tick(10) {
        return;
    }
    for worker in &workers.0 {
        if sw
            .entry_ref(worker.0)
            .unwrap()
            .get_component::<Desire<Work>>()
            .is_err()
        {
            cbuf.add_component(worker.0, Desire::<Work>::new(Work::new(farm.building, 0.0)))
        }
    }
}

#[system(par_for_each)]
pub fn farm_cereal(
    #[resource] time: &GameTime,
    #[resource] cbuf: &ParCommandBuffer,
    me: &Entity,
    farm: &mut Farm,
    workers: &Workers,
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
}
