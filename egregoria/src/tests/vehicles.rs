#![cfg(test)]

use crate::engine_interaction::WorldCommands;
use crate::map_dynamic::{Itinerary, ParkingManagement};
use crate::vehicles::{spawn_parked_vehicle, unpark, VehicleKind};
use crate::Egregoria;
use common::logger::MyLog;
use common::GameTime;
use geom::vec2;
use map_model::{LanePatternBuilder, Map, MapProject, PathKind, ProjectKind};

#[test]
fn test_car_simple() {
    MyLog::init();

    let mut g = Egregoria::empty();
    let mut sched = Egregoria::schedule();

    let (i, _) = g
        .write::<Map>()
        .make_connection(
            MapProject {
                kind: ProjectKind::Ground,
                pos: vec2(0.0, 0.0),
            },
            MapProject {
                kind: ProjectKind::Ground,
                pos: vec2(100.0, 0.0),
            },
            None,
            &LanePatternBuilder::default().build(),
        )
        .unwrap();

    let (_, _) = g
        .write::<Map>()
        .make_connection(
            MapProject {
                kind: ProjectKind::Inter(i),
                pos: vec2(0.0, 0.0),
            },
            MapProject {
                kind: ProjectKind::Ground,
                pos: vec2(100.0, 50.0),
            },
            None,
            &LanePatternBuilder::default().build(),
        )
        .unwrap();
    let car = spawn_parked_vehicle(&mut g, VehicleKind::Car, vec2(0.0, 0.0)).unwrap();
    unpark(&mut g, car);

    let pos = g.pos(car.0).unwrap();

    let spot_id = g
        .write::<ParkingManagement>()
        .reserve_near(vec2(50.0, 50.0), &*g.map())
        .unwrap();
    let spot = *g.map().parking.get(spot_id).unwrap();
    let end_lane = g.map().parking_to_drive(spot_id).unwrap();
    let end_pos = g.map().lanes()[end_lane]
        .points
        .project(spot.trans.position());

    let itin = Itinerary::route(pos, end_pos, &*g.read::<Map>(), PathKind::Vehicle).unwrap();
    *g.comp_mut::<Itinerary>(car.0).unwrap() = itin;

    for _ in 0..1000 {
        g.tick(&mut sched, &WorldCommands::default());

        if g.comp::<Itinerary>(car.0)
            .unwrap()
            .has_ended(g.read::<GameTime>().timestamp)
        {
            return;
        }
    }

    panic!("car has not arrived after 1000 ticks.")
}
