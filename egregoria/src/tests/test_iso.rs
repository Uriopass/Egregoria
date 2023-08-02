use crate::engine_interaction::WorldCommand;
use crate::init::init;
use crate::map::{Map, MapProject, ProjectKind};
use crate::utils::scheduler::SeqSchedule;
use crate::utils::time::Tick;
use crate::World;
use crate::{Egregoria, Replay};
use common::logger::MyLog;
use common::saveload::Encoder;

static REPLAY: &'static [u8] = include_bytes!("world_replay.json");

fn check_coherent(map: &Map, proj: MapProject) {
    match proj.kind {
        ProjectKind::Inter(i) => {
            assert!(map.intersections.get(i).unwrap().pos.distance(proj.pos) < 5.0)
        }
        ProjectKind::Road(_) => {
            //assert!(map.roads.get(r).unwrap().points.project_dist(proj.pos) < 5.0)
        }
        _ => {}
    }
}

fn check_eq(w1: &World, w2: &World) -> bool {
    for (c1, c2) in w1.entities().zip(w2.entities()) {
        if c1 != c2 {
            println!("{:?} {:?}", c1, c2);
            return false;
        }
    }
    true
}

//#[test] // uncomment when slotmap has been forked
fn test_world_survives_serde() {
    init();
    MyLog::init();

    let replay: Replay = common::saveload::JSONPretty::decode(REPLAY).unwrap();
    let (mut goria, mut loader) = Egregoria::from_replay(replay.clone());
    let (mut goria2, mut loader2) = Egregoria::from_replay(replay);
    let mut s = SeqSchedule::default();

    let mut idx = 0;
    while !loader.advance_tick(&mut goria, &mut s) {
        loader2.advance_tick(&mut goria2, &mut s);

        let next_idx = idx
            + loader.replay.commands[idx..]
                .iter()
                .enumerate()
                .find_map(|(i, (t, _))| if *t > loader.pastt { Some(i) } else { None })
                .unwrap_or(idx);
        for (tick, command) in &loader.replay.commands[idx..next_idx] {
            match command {
                WorldCommand::MapMakeConnection { from, to, .. } => {
                    println!("{:?} {:?}", tick, command);
                    let map = goria.map();

                    check_coherent(&*map, *from);
                    println!("ho");
                    check_coherent(&*map, *to);
                }
                _ => {}
            }
        }

        idx = next_idx;

        if goria.read::<Tick>().0 % 100 != 0 && goria.read::<Tick>().0 <= 4130 {
            continue;
        }

        println!(
            "--- tick {} ({}/{})",
            goria.read::<Tick>().0,
            loader.pastt.0,
            loader.replay.commands.last().unwrap().0 .0
        );

        let ser = common::saveload::Bincode::encode(&goria).unwrap();
        let mut deser: Egregoria = common::saveload::Bincode::decode(&ser).unwrap();

        deser.assert_equal(&goria);
        deser.assert_equal(&goria2);

        std::mem::swap(&mut deser, &mut goria2);
    }

    goria.save_to_disk("world2");
}
