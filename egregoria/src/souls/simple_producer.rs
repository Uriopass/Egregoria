use crate::economy::{Commodity, JobApplication, Market};
use crate::map_dynamic::{BuildingInfos, Router};
use crate::pedestrians::spawn_pedestrian;
use crate::souls::desire::{Home, Work};
use crate::utils::rand_provider::RandProvider;
use crate::{Egregoria, SoulID};
use common::{GameInstant, GameTime};
use map_model::{BuildingID, Map};

pub struct Farm {
    building: BuildingID,
    last_harvest: GameInstant,
}

pub fn farm_soul<I: Commodity, O: Commodity>(
    goria: &mut Egregoria,
    farm: BuildingID,
) -> Option<()> {
    let farmpos = goria.read::<Map>().buildings()[farm].door_pos;

    let e = goria.world.push((Farm {
        building: farm,
        last_harvest: goria.read::<GameTime>().instant(),
    },));

    let soul = SoulID(e);

    goria
        .write::<Market<JobApplication>>()
        .sell(soul, farmpos, 1);
    goria.write::<BuildingInfos>().set_owner(farm, soul);

    let offset = goria.write::<RandProvider>().random::<f32>() * 0.5;
    let mut e = goria.world.entry(human.0).unwrap();

    e.add_component(Desire::new(Work::new(work, offset)));
    e.add_component(Desire::new(Home::new(house, offset)));
    e.add_component(Router::new(car));
    Some(())
}
