use crate::init::init;
use crate::map::{LanePatternBuilder, Map, MapProject, ProjectKind};
use crate::utils::scheduler::SeqSchedule;
use crate::World;
use crate::{Replay, Simulation};
use common::saveload::{Bincode, Encoder, JSONPretty};
use geom::vec3;
use quickcheck::{Arbitrary, Gen, TestResult};

static REPLAY: &[u8] = include_bytes!("world_replay.json");

fn check_coherent(map: &Map, proj: MapProject) {
    match proj.kind {
        ProjectKind::Intersection(i) => {
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
    let mut q = quickcheck::QuickCheck::new().tests(100);
    q.quickcheck(
        (|vals: Vec<(MapAction, u32, F3201, F3201)>| -> TestResult {
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
                                kind: ProjectKind::Intersection(i),
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
                                pos: m2.intersections[i].pos,
                                kind: ProjectKind::Intersection(i),
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
                                    kind: ProjectKind::Intersection(i1),
                                },
                                MapProject {
                                    pos: m.intersections[i2].pos,
                                    kind: ProjectKind::Intersection(i2),
                                },
                                None,
                                &LanePatternBuilder::new().build(),
                            );
                            m2.make_connection(
                                MapProject {
                                    pos: m2.intersections[i1].pos,
                                    kind: ProjectKind::Intersection(i1),
                                },
                                MapProject {
                                    pos: m2.intersections[i2].pos,
                                    kind: ProjectKind::Intersection(i2),
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

            let v = Bincode::encode(&m).unwrap() == Bincode::encode(&m2).unwrap();
            if !v {
                let m_enc = unsafe { String::from_utf8_unchecked(JSONPretty::encode(&m).unwrap()) };
                let m2_enc =
                    unsafe { String::from_utf8_unchecked(JSONPretty::encode(&m2).unwrap()) };
                let diff = diff::lines(&m_enc, &m2_enc);
                let mut diff_str = String::new();
                for line in diff {
                    match line {
                        diff::Result::Left(l) => diff_str.push_str(&format!("- {}\n", l)),
                        diff::Result::Both(l, _) => diff_str.push_str(&format!("  {}\n", l)),
                        diff::Result::Right(r) => diff_str.push_str(&format!("+ {}\n", r)),
                    }
                }

                TestResult::error(diff_str)
            } else {
                TestResult::passed()
            }
        }) as fn(_) -> TestResult,
    );
}

#[test]
fn test_world_survives_serde() {
    init();
    //common::logger::MyLog::init();

    let replay: Replay = JSONPretty::decode(REPLAY).unwrap();
    let mut s = SeqSchedule::default();

    let mut check_size = 1024;
    let mut check_start = 3;

    'main: loop {
        if check_size == 0 {
            break;
        }
        let (mut sim, mut loader) = Simulation::from_replay(replay.clone());
        let (mut sim2, mut loader2) = Simulation::from_replay(replay.clone());

        while !loader.advance_tick(&mut sim, &mut s) {
            loader2.advance_tick(&mut sim2, &mut s);

            let tick = sim.get_tick();
            if tick < check_start || tick % check_size != 0 {
                continue;
            }
            println!(
                "--- tick {} ({}/{})",
                sim.get_tick(),
                loader.pastt.0,
                loader.replay.last_tick_recorded.0
            );

            let ser = common::saveload::Bincode::encode(&sim).unwrap();
            let mut deser: Simulation = common::saveload::Bincode::decode(&ser).unwrap();

            if !sim.is_equal(&sim2) {
                println!("not equal sim+sim2");
                sim.save_to_disk("world");
                sim2.save_to_disk("world2");
                check_start = tick - check_size;
                check_size = check_size / 2;
                continue 'main;
            }
            if !deser.is_equal(&sim) {
                println!("not equal sim");
                deser.save_to_disk("world");
                sim.save_to_disk("world2");
                check_start = tick - check_size;
                check_size = check_size / 2;
                continue 'main;
            }
            if !deser.is_equal(&sim2) {
                println!("not equal sim2");
                deser.save_to_disk("world");
                sim2.save_to_disk("world2");
                check_start = tick - check_size;
                check_size = check_size / 2;
                continue 'main;
            }

            std::mem::swap(&mut deser, &mut sim2);
        }

        break;
    }
}
