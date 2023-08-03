use crate::init::init;
use crate::map::{LanePatternBuilder, Map, MapProject, ProjectKind};
use crate::utils::scheduler::SeqSchedule;
use crate::utils::time::Tick;
use crate::World;
use crate::{Egregoria, Replay};
use common::logger::MyLog;
use common::saveload::{Bincode, Encoder};
use geom::vec3;
use quickcheck::{Arbitrary, Gen};

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

#[derive(Debug, Copy, Clone)]
struct F3201(f32);

impl Arbitrary for F3201 {
    fn arbitrary(g: &mut Gen) -> Self {
        let v = <u32 as Arbitrary>::arbitrary(g);
        F3201(v as f32 / u32::MAX as f32)
    }
}

#[derive(Debug, Copy, Clone)]
enum MapAction {
    AddInter,
    TwoInter,
    RemoveRoad,
    SplitRoad,
    Serde,
}

impl Arbitrary for MapAction {
    fn arbitrary(g: &mut Gen) -> Self {
        *g.choose(&[
            MapAction::AddInter,
            MapAction::TwoInter,
            MapAction::SplitRoad,
            MapAction::Serde,
        ])
        .unwrap()
    }
}

#[test]
fn quickcheck_map_ser() {
    let mut q = quickcheck::QuickCheck::new();
    q.quickcheck(
        (|vals: Vec<(MapAction, u32, F3201, F3201)>| -> bool {
            let mut m = Map::empty();
            let mut m2 = Map::empty();

            m.make_connection(
                MapProject {
                    pos: vec3(0.0, 0.0, 0.0),
                    kind: ProjectKind::Ground,
                },
                MapProject {
                    pos: vec3(30.0, 0.0, 0.0),
                    kind: ProjectKind::Ground,
                },
                None,
                &LanePatternBuilder::new().build(),
            );

            m2.make_connection(
                MapProject {
                    pos: vec3(0.0, 0.0, 0.0),
                    kind: ProjectKind::Ground,
                },
                MapProject {
                    pos: vec3(30.0, 0.0, 0.0),
                    kind: ProjectKind::Ground,
                },
                None,
                &LanePatternBuilder::new().build(),
            );

            for (action, r, x, y) in vals {
                match action {
                    MapAction::AddInter => {
                        let i = m
                            .intersections
                            .iter()
                            .nth(r as usize % m.intersections.len())
                            .unwrap()
                            .0;
                        m.make_connection(
                            MapProject {
                                pos: m.intersections[i].pos,
                                kind: ProjectKind::Inter(i),
                            },
                            MapProject {
                                pos: vec3(x.0 * 500.0, y.0 * 500.0, 0.0),
                                kind: ProjectKind::Ground,
                            },
                            None,
                            &LanePatternBuilder::new().build(),
                        );
                        m2.make_connection(
                            MapProject {
                                pos: m.intersections[i].pos,
                                kind: ProjectKind::Inter(i),
                            },
                            MapProject {
                                pos: vec3(x.0 * 500.0, y.0 * 500.0, 0.0),
                                kind: ProjectKind::Ground,
                            },
                            None,
                            &LanePatternBuilder::new().build(),
                        );
                    }
                    MapAction::TwoInter => {
                        let i1 = m
                            .intersections
                            .iter()
                            .nth(r as usize % m.intersections.len())
                            .unwrap()
                            .0;
                        let i2 = m
                            .intersections
                            .iter()
                            .nth(r as usize % m.intersections.len())
                            .unwrap()
                            .0;
                        if i1 != i2 {
                            m.make_connection(
                                MapProject {
                                    pos: m.intersections[i1].pos,
                                    kind: ProjectKind::Inter(i1),
                                },
                                MapProject {
                                    pos: m.intersections[i2].pos,
                                    kind: ProjectKind::Inter(i2),
                                },
                                None,
                                &LanePatternBuilder::new().build(),
                            );
                            m2.make_connection(
                                MapProject {
                                    pos: m2.intersections[i1].pos,
                                    kind: ProjectKind::Inter(i1),
                                },
                                MapProject {
                                    pos: m2.intersections[i2].pos,
                                    kind: ProjectKind::Inter(i2),
                                },
                                None,
                                &LanePatternBuilder::new().build(),
                            );
                        }
                    }
                    MapAction::RemoveRoad => {
                        if m.roads.len() > 3 {
                            // remove randomly
                            let r = m.roads.iter().nth(r as usize % m.roads.len()).unwrap().0;

                            m.remove_road(r);
                            m2.remove_road(r);
                        }
                    }
                    MapAction::SplitRoad => {
                        let r = m.roads.iter().nth(r as usize % m.roads.len()).unwrap().0;
                        let p = m.roads[r].points.length();
                        m.make_connection(
                            MapProject {
                                pos: m.roads[r].points.point_along(p * 0.5),
                                kind: ProjectKind::Road(r),
                            },
                            MapProject {
                                pos: vec3(x.0 * 500.0, y.0 * 500.0, 0.0),
                                kind: ProjectKind::Ground,
                            },
                            None,
                            &LanePatternBuilder::new().build(),
                        );
                        m2.make_connection(
                            MapProject {
                                pos: m2.roads[r].points.point_along(p * 0.5),
                                kind: ProjectKind::Road(r),
                            },
                            MapProject {
                                pos: vec3(x.0 * 500.0, y.0 * 500.0, 0.0),
                                kind: ProjectKind::Ground,
                            },
                            None,
                            &LanePatternBuilder::new().build(),
                        );
                    }
                    MapAction::Serde => {
                        let v = Bincode::encode(&m).unwrap();
                        let m3: Map = Bincode::decode(&v).unwrap();
                        m = m3;
                    }
                }
            }

            Bincode::encode(&m).unwrap() == Bincode::encode(&m2).unwrap()
        }) as fn(_) -> bool,
    );
}

//#[test]
fn test_world_survives_serde() {
    init();
    MyLog::init();

    let replay: Replay = common::saveload::JSONPretty::decode(REPLAY).unwrap();
    let (mut goria, mut loader) = Egregoria::from_replay(replay.clone());
    let (mut goria2, mut loader2) = Egregoria::from_replay(replay);
    let mut s = SeqSchedule::default();

    //let mut idx = 0;
    while !loader.advance_tick(&mut goria, &mut s) {
        loader2.advance_tick(&mut goria2, &mut s);

        /*
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

        idx = next_idx;*/

        let tick = goria.read::<Tick>().0;
        if tick % 1000 != 0 || (tick < 7840) {
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

        if !deser.is_equal(&goria) {
            println!("not equal");
            deser.save_to_disk("world");
            goria.save_to_disk("world2");
            assert!(false);
        }
        if !deser.is_equal(&goria2) {
            println!("not equal");
            deser.save_to_disk("world");
            goria2.save_to_disk("world2");
            assert!(false);
        }

        std::mem::swap(&mut deser, &mut goria2);
    }

    goria.save_to_disk("world2");
}
