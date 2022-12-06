use crate::map::{BuildingID, BuildingKind, Map, PathKind};
use crate::map_dynamic::{BuildingInfos, DispatchKind, DispatchQueryTarget, Dispatcher, Itinerary};
use crate::utils::time::GameTime;
use crate::vehicles::trains::TrainID;
use crate::{Egregoria, ParCommandBuffer, Selectable, SoulID};
use geom::Transform;
use hecs::World;
use resources::Resources;
use serde::{Deserialize, Serialize};

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
    pub building: BuildingID,
    pub trains: Vec<(TrainID, FreightTrainState)>,
    pub waiting_cargo: u32,
}

pub fn freight_station_soul(goria: &mut Egregoria, building: BuildingID) -> Option<SoulID> {
    let map = goria.map();

    let f = FreightStation {
        building,
        trains: Vec::with_capacity(MAX_TRAINS_PER_STATION),
        waiting_cargo: 0,
    };
    let b = map.buildings.get(building)?;

    let height = b.height;
    let obb = b.obb;
    let pos = obb.center();
    let [w2, h2] = obb.axis().map(|x| x.mag2());

    drop(map);

    let soul = SoulID(goria.world.spawn((
        f,
        Transform::new(pos.z(height)),
        Selectable {
            radius: w2.max(h2).sqrt() * 0.5,
        },
    )));

    goria.write::<BuildingInfos>().set_owner(building, soul);

    Some(soul)
}

pub fn freight_station_system(world: &mut World, resources: &mut Resources) {
    let cbuf = resources.get::<ParCommandBuffer>().unwrap();
    let mut dispatch = resources.get_mut::<Dispatcher>().unwrap();
    let map = resources.get::<Map>().unwrap();
    let time = resources.get::<GameTime>().unwrap();

    let mut trainqry = world.query::<(&Transform, &mut Itinerary)>();
    let mut train = trainqry.view();

    for (me, (pos, soul)) in world
        .query::<(&Transform, &mut FreightStation)>()
        .into_iter()
    {
        if !map.buildings.contains_key(soul.building) {
            cbuf.kill(me);
            continue;
        }

        // update our trains, and remove the ones that are done
        let mut to_clean = vec![];
        for (trainid, state) in &mut soul.trains {
            let Some((tpos, itin)) = train.get_mut(trainid.0) else {
                to_clean.push(*trainid);
                continue
            };
            match state {
                FreightTrainState::Arriving => {
                    if itin.has_ended(0.0) {
                        *state = FreightTrainState::Loading;
                        soul.waiting_cargo -= 10;
                        *itin = Itinerary::wait_until(time.timestamp + 10.0);
                    }
                }
                FreightTrainState::Loading => {
                    if itin.has_ended(time.timestamp) {
                        let ext = map.bkinds.get(&BuildingKind::ExternalTrading).unwrap()[0];
                        let bpos = map.buildings[ext].obb.center().z(0.0);

                        *itin = if let Some(r) =
                            Itinerary::route(tpos.position, bpos, &map, PathKind::Rail)
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
            soul.trains.retain(|x| x.0 != v);
            dispatch.free(DispatchKind::FretTrain, v.0)
        }

        // If enough goods are waiting, query for a train to take them to the external trading station
        if soul.trains.len() >= MAX_TRAINS_PER_STATION {
            continue;
        }
        if soul.waiting_cargo < 10 {
            continue;
        }
        let Some(trainid) = dispatch.query(
            &map,
            me,
            DispatchKind::FretTrain,
            DispatchQueryTarget::Pos(pos.position),
        ) else {
            continue;
        };
        let trainid = TrainID(trainid);

        let (tpos, titin) = train.get_mut(trainid.0).unwrap();

        *titin = unwrap_or!(
            Itinerary::route(tpos.position, pos.position, &map, PathKind::Rail,),
            continue
        );

        soul.trains.push((trainid, FreightTrainState::Arriving));
    }
}

#[cfg(test)]
mod tests {
    use crate::map::BuildingGen;
    use crate::map_dynamic::BuildingInfos;
    use crate::souls::human::{spawn_human, HumanDecisionKind};
    use crate::tests::TestCtx;
    use crate::{BuildingKind, FreightStation, HumanDecision, WorldCommand};
    use geom::{vec2, vec3, OBB};

    #[test]
    fn test_deliver_to_freight_station_incrs_station() {
        let mut test = TestCtx::new();

        test.build_roads(&[vec3(0., 0., 0.), vec3(100., 0., 0.)]);
        let house = test.build_house_near(vec2(50.0, 50.0));
        let human = spawn_human(&mut test.g, house).unwrap();

        test.apply(&[WorldCommand::MapBuildSpecialBuilding(
            OBB::new(vec2(50.0, 50.0), vec2(1.0, 0.0), 5.0, 5.0),
            BuildingKind::RailFretStation,
            BuildingGen::NoWalkway {
                door_pos: vec2(50.0, 50.0),
            },
            vec![],
        )]);
        test.tick();

        let station = test
            .g
            .map()
            .buildings()
            .iter()
            .find(|(_, b)| matches!(b.kind, BuildingKind::RailFretStation))
            .unwrap()
            .0;

        test.g.comp_mut::<HumanDecision>(human.0).unwrap().kind =
            HumanDecisionKind::DeliverAtBuilding(station);

        let binfos = test.g.read::<BuildingInfos>();
        let stationsoul = binfos.owner(station).unwrap();
        drop(binfos);

        for _ in 0..100 {
            test.tick();

            if test
                .g
                .comp::<FreightStation>(stationsoul.0)
                .unwrap()
                .waiting_cargo
                == 1
            {
                return;
            }
        }

        assert!(false);
    }
}
