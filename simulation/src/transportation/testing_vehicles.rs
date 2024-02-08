use crate::map::{Map, PathKind};
use crate::map_dynamic::Itinerary;
use crate::utils::resources::Resources;
use crate::{VehicleID, World};
use common::scroll::BTreeSetScroller;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Default, Serialize, Deserialize)]
pub struct RandomVehicles {
    pub vehicles: BTreeSet<VehicleID>,
    pub vehicle_scroller: BTreeSetScroller<VehicleID>,
}

pub fn random_vehicles_update(world: &mut World, res: &mut Resources) {
    profiling::scope!("transportation::random_vehicles_update");

    let rv = &mut *res.write::<RandomVehicles>();
    let map = res.read::<Map>();

    let mut to_kill = Vec::new();

    let tick = res.tick();

    for &v_id in rv.vehicle_scroller.iter_looped(&rv.vehicles).take(100) {
        let v = match world.vehicles.get_mut(v_id) {
            Some(x) => x,
            None => {
                to_kill.push(v_id);
                continue;
            }
        };

        if !(v.it.has_ended(0.0) || v.it.is_wait_for_reroute().is_some()) {
            continue;
        }
        let rng = common::hash_u64((tick.0, v_id));

        if let Some(it) = Itinerary::random_route(rng, v.trans.pos, tick, &map, PathKind::Vehicle) {
            v.it = it;
        }
    }

    for v in to_kill {
        rv.vehicles.remove(&v);
    }
}
