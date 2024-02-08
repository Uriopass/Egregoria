use serde::{Deserialize, Serialize};

use geom::Transform;
use prototypes::{FreightStationPrototypeID, GameTime};

use crate::map::{BuildingID, Map, PathKind};
use crate::map_dynamic::{
    BuildingInfos, DispatchID, DispatchKind, DispatchQueryTarget, Dispatcher, Itinerary,
};
use crate::utils::resources::Resources;
use crate::world::{FreightStationEnt, FreightStationID, TrainID};
use crate::World;
use crate::{ParCommandBuffer, Simulation, SoulID};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Inspect)]
pub enum FreightTrainState {
    /// The train is coming to the station
    Arriving,
    /// The train is waiting for the station to load goods
    Loading,
    /// The train is going to the destination
    Moving,
}

const MAX_TRAINS_PER_STATION: usize = 2;

/// A freight train station
/// A component that identifies freight station souls, managing freight station logic
/// and the freight trains that are associated with them.
#[derive(Serialize, Deserialize, Inspect)]
pub struct FreightStation {
    pub proto: FreightStationPrototypeID,
    pub building: BuildingID,
    pub trains: Vec<(TrainID, FreightTrainState)>,
    pub waiting_cargo: u32,
    pub wanted_cargo: u32,
}

pub fn freight_station_soul(
    sim: &mut Simulation,
    building: BuildingID,
    proto: FreightStationPrototypeID,
) -> Option<FreightStationID> {
    let map = sim.map();

    let f = FreightStation {
        proto,
        building,
        trains: Vec::with_capacity(MAX_TRAINS_PER_STATION),
        waiting_cargo: 0,
        wanted_cargo: 0,
    };
    let b = map.buildings.get(building)?;

    let height = b.height;
    let obb = b.obb;
    let pos = obb.center();
    let axis = obb.axis();

    drop(map);

    let id = sim.world.insert(FreightStationEnt {
        f,
        trans: Transform::new_dir(pos.z(height), axis[1].z(0.0).normalize()),
    });

    sim.write::<BuildingInfos>()
        .set_owner(building, SoulID::FreightStation(id));

    Some(id)
}

pub fn freight_station_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("souls::freight_station_system");
    let cbuf = resources.read::<ParCommandBuffer<FreightStationEnt>>();
    let mut dispatch = resources.write::<Dispatcher>();
    let map = resources.read::<Map>();
    let time = resources.read::<GameTime>();
    let tick = time.tick;

    for (me, f) in world.freight_stations.iter_mut() {
        let pos = f.trans;
        let station = &mut f.f;
        if !map.buildings.contains_key(station.building) {
            cbuf.kill(me);
            continue;
        }

        // update our trains, and remove the ones that are done
        let mut to_clean = vec![];
        for (trainid, state) in &mut station.trains {
            let Some(train) = world.trains.get_mut(*trainid) else {
                to_clean.push(*trainid);
                continue;
            };
            let itin = &mut train.it;

            match state {
                FreightTrainState::Arriving => {
                    if itin.has_ended(0.0) {
                        *state = FreightTrainState::Loading;
                        station.waiting_cargo = station.waiting_cargo.saturating_sub(100);
                        station.wanted_cargo = station.wanted_cargo.saturating_sub(100);
                        *itin = Itinerary::wait_until(time.timestamp + 10.0);
                    }
                }
                FreightTrainState::Loading => {
                    if itin.has_ended(time.timestamp) {
                        let ext = *map.external_train_stations.first().unwrap();
                        let bpos = map.buildings[ext].obb.center().z(0.0);

                        *itin = if let Some(r) =
                            Itinerary::route(tick, train.trans.pos, bpos, &map, PathKind::Rail)
                        {
                            r
                        } else {
                            Itinerary::wait_until(time.timestamp + 10.0);
                            continue;
                        };
                        *state = FreightTrainState::Moving;
                    }
                }
                FreightTrainState::Moving => {
                    if itin.has_ended(time.timestamp) {
                        to_clean.push(*trainid);
                    }
                }
            }
        }
        for v in to_clean {
            station.trains.retain(|x| x.0 != v);
            dispatch.free(v)
        }

        // If enough goods are waiting, query for a train to take them to the external trading station
        if station.trains.len() >= MAX_TRAINS_PER_STATION {
            continue;
        }
        if station.waiting_cargo + station.wanted_cargo < 10 {
            continue;
        }

        let destination = pos.pos + pos.dir * 75.0 - pos.dir.perp_up() * 40.0;

        let Some(DispatchID::FreightTrain(trainid)) = dispatch.query(
            &map,
            DispatchKind::FreightTrain,
            DispatchQueryTarget::Pos(destination),
        ) else {
            continue;
        };

        let train = world.trains.get_mut(trainid).unwrap();

        train.it = unwrap_or!(
            Itinerary::route(tick, train.trans.pos, destination, &map, PathKind::Rail,),
            continue
        );

        station.trains.push((trainid, FreightTrainState::Arriving));
    }
}

#[cfg(test)]
mod tests {
    use geom::{vec2, vec3, OBB};
    use prototypes::{BuildingGen, FreightStationPrototypeID};

    use crate::map_dynamic::BuildingInfos;
    use crate::souls::human::{spawn_human, HumanDecisionKind};
    use crate::tests::TestCtx;
    use crate::{BuildingKind, SoulID, WorldCommand};

    #[test]
    fn test_deliver_to_freight_station_incrs_station() {
        let mut test = TestCtx::new();

        test.build_roads(&[vec3(0., 0., 0.), vec3(100., 0., 0.)]);
        let house = test.build_house_near(vec2(50.0, 50.0));
        let human = spawn_human(&mut test.g, house).unwrap();

        test.apply(&[WorldCommand::MapBuildSpecialBuilding {
            pos: OBB::new(vec2(50.0, 50.0), vec2(1.0, 0.0), 5.0, 5.0),
            kind: BuildingKind::RailFreightStation(FreightStationPrototypeID::new(
                "freight-station",
            )),
            gen: BuildingGen::NoWalkway {
                door_pos: vec2(50.0, 50.0),
            },
            zone: None,
            connected_road: None,
        }]);
        test.tick();

        let station = test
            .g
            .map()
            .buildings()
            .iter()
            .find(|(_, b)| matches!(b.kind, BuildingKind::RailFreightStation(_)))
            .unwrap()
            .0;

        test.g
            .world_mut_unchecked()
            .humans
            .get_mut(human)
            .unwrap()
            .decision
            .kind = HumanDecisionKind::DeliverAtBuilding(station);

        let binfos = test.g.read::<BuildingInfos>();
        let SoulID::FreightStation(stationsoul) = binfos.owner(station).unwrap() else {
            panic!()
        };
        drop(binfos);

        for _ in 0..100 {
            test.tick();

            if test.g.get(stationsoul).unwrap().f.waiting_cargo == 1 {
                return;
            }
        }

        panic!("should have delivered to freight station")
    }
}
