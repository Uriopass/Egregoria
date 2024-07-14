use crate::map::{LaneID, LaneKind, TraverseDirection};
use crate::utils::resources::Resources;
use crate::world::{TrainID, VehicleID};
use crate::{Map, World};
use derive_more::From;
use geom::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

/// How precise the dispatcher is. Caches dispatchable entities's positions and relation to map but only in precision circle.
/// So if a dispatchable entity moves less than the precision, nothing will be updated.
const PRECISION_RADIUS: f32 = 5.0;
const PRECISION_RADIUS_2: f32 = PRECISION_RADIUS * PRECISION_RADIUS;

/// Dispatcher is used to query for the closest networked entity matching a condition
/// For example:
/// - A rail freight station will query for the closest train to it that is not already used by another station
/// - A factory will query for a truck to deliver goods
/// - A hospital will query for the closest injured person
#[derive(Default, Serialize, Deserialize)]
pub struct Dispatcher {
    dispatches: BTreeMap<DispatchKind, DispatchOne>,
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, Eq, PartialEq, Serialize, Deserialize, From)]
pub enum DispatchID {
    FreightTrain(TrainID),
    SmallTruck(VehicleID),
}

impl From<DispatchID> for DispatchKind {
    fn from(id: DispatchID) -> Self {
        match id {
            DispatchID::FreightTrain(_) => DispatchKind::FreightTrain,
            DispatchID::SmallTruck(_) => DispatchKind::SmallTruck,
        }
    }
}

/// Dispatcher specialized to one kind
#[derive(Serialize, Deserialize)]
struct DispatchOne {
    positions: BTreeMap<DispatchID, DispatchPosition>,
    lanes: BTreeMap<LaneID, Vec<DispatchID>>,
    reserved_by: BTreeSet<DispatchID>,
    lanekind: LaneKind,
}

#[derive(Serialize, Deserialize)]
struct DispatchPosition {
    lane: LaneID,
    pos: Vec3,
    dist_along: f32,
}

/// DispatchKind is a component that is added to entities that can be dispatched
/// Usually constant.
#[derive(Serialize, Deserialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Inspect)]
pub enum DispatchKind {
    FreightTrain,
    SmallTruck,
}

impl DispatchKind {
    pub fn lane_kind(self) -> LaneKind {
        match self {
            DispatchKind::FreightTrain => LaneKind::Rail,
            DispatchKind::SmallTruck => LaneKind::Driving,
        }
    }
}

/// DispatchQueryTarget is the target that can be queried to the dispatcher
#[derive(Copy, Clone)]
pub enum DispatchQueryTarget {
    Pos(Vec3),
    Lane(LaneID),
}

impl Dispatcher {
    /// Updates the dispatcher cache about the dispatachable entities to know where they are relative
    /// to the map, so that queries can be answered quickly
    pub fn update(&mut self, map: &Map, world: &World) {
        let disp_trains = self
            .dispatches
            .entry(DispatchKind::FreightTrain)
            .or_insert_with(|| DispatchOne::new(DispatchKind::FreightTrain.lane_kind()));

        world.trains.iter().for_each(|(ent, train)| {
            disp_trains.register(DispatchID::FreightTrain(ent), map, train.trans.pos);
        });

        /*
        let disp_trucks = self
            .dispatches
            .entry(DispatchKind::SmallTruck)
            .or_insert_with(|| DispatchOne::new(DispatchKind::SmallTruck.lane_kind()));

        world.vehicles.iter().for_each(|(ent, truck)| {
            disp_trucks.register(DispatchID::Truck(ent), map, truck.trans.position);
        })*/
    }

    /// Frees the entity as it is no longer used
    /// For example if a train is no longer used by a station, it should be freed so that other stations can use it
    /// It should be re-added to the cache at the next update iteration
    pub fn free(&mut self, ent: impl Into<DispatchID>) {
        let ent: DispatchID = ent.into();
        let kind: DispatchKind = ent.into();
        let Some(disp) = self.dispatches.get_mut(&kind) else {
            return;
        };
        disp.reserved_by.remove(&ent);
    }

    pub fn unregister(&mut self, id: DispatchID) {
        let kind = id.into();
        let Some(disp) = self.dispatches.get_mut(&kind) else {
            return;
        };
        disp.unregister(id);
    }

    /// Reserves an entity that is closest to the target (if it is found) and returns it
    /// it takes `me` as an argument so that if `me` is killed, the reservation is cancelled
    /// If no entity is found, returns None
    pub fn query(
        &mut self,
        map: &Map,
        kind: DispatchKind,
        target: DispatchQueryTarget,
    ) -> Option<DispatchID> {
        let disp = self.dispatches.get_mut(&kind)?;
        let best_ent = disp.query(map, kind, target)?;
        disp.reserve(best_ent);
        Some(best_ent)
    }
}

impl DispatchOne {
    fn new(lanekind: LaneKind) -> Self {
        Self {
            positions: Default::default(),
            lanes: Default::default(),
            reserved_by: Default::default(),
            lanekind,
        }
    }

    fn register(&mut self, id: DispatchID, map: &Map, pos: Vec3) {
        let ent = self.positions.entry(id);

        let lanekind = self.lanekind;
        let find_lane = move || map.nearest_lane(pos, lanekind, Some(50.0));

        match ent {
            Entry::Vacant(v) => {
                let Some(n) = find_lane() else { return };
                let newl = &map.lanes[n];
                let proj = newl.points.project(pos);

                self.lanes.entry(n).or_default().push(id);
                v.insert(DispatchPosition {
                    lane: n,
                    pos,
                    dist_along: newl.points.length_at_proj(proj),
                });
            }
            Entry::Occupied(mut o) => {
                let dp = o.get_mut();

                if dp.pos.distance2(pos) < PRECISION_RADIUS_2 {
                    return;
                }

                if let Some(l) = map.lanes().get(dp.lane) {
                    let projected = l.points.project(pos);
                    if projected.distance2(pos) < PRECISION_RADIUS_2 {
                        dp.dist_along = l.points.length_at_proj(projected);
                        return;
                    }
                }

                let Some(n) = find_lane() else { return };
                self.lanes.get_mut(&dp.lane).unwrap().retain(|e| *e != id);
                self.lanes.entry(n).or_default().push(id);

                let newl = &map.lanes[n];

                let projected = newl.points.project(pos);
                *dp = DispatchPosition {
                    lane: n,
                    pos,
                    dist_along: newl.points.length_at_proj(projected),
                };
            }
        }
    }

    fn reserve(&mut self, id: DispatchID) {
        self.reserved_by.insert(id);
        let Some(pos) = self.positions.remove(&id) else {
            log::error!("Dispatcher: trying to reserve an entity that is not in the cache");
            return;
        };
        self.lanes.get_mut(&pos.lane).unwrap().retain(|e| *e != id);
    }

    pub fn unregister(&mut self, id: DispatchID) {
        let Some(pos) = self.positions.remove(&id) else {
            return;
        };
        self.reserved_by.remove(&id);
        self.lanes.get_mut(&pos.lane).unwrap().retain(|e| *e != id);
    }

    /// Finds an entity that is closest to the target and returns it
    /// If no entity is found, returns None
    pub fn query(
        &mut self,
        map: &Map,
        kind: DispatchKind,
        target: DispatchQueryTarget,
    ) -> Option<DispatchID> {
        // todo: handle the case where there are few entities in the cache
        // todo: probably some kind of astar on good candidates

        let mut start_along = f32::MAX;

        if self.positions.is_empty() {
            return None;
        }

        let target_lane = match target {
            DispatchQueryTarget::Pos(pos) => {
                let lid = map.nearest_lane(pos, kind.lane_kind(), Some(50.0))?;
                let lane = map.lanes().get(lid)?;
                let proj = lane.points.project(pos);
                start_along = lane.points.length_at_proj(proj);
                lid
            }
            DispatchQueryTarget::Lane(lane) => {
                #[allow(clippy::question_mark)]
                if map.lanes().get(lane).is_none() {
                    return None;
                }
                lane
            }
        };

        let mut best_dist = f32::MAX;
        let mut best_ent = None;

        // do a backward breadth first search, looking for lanes with matching entities
        for lane in pathfinding::directed::bfs::bfs_reach(target_lane, move |&lid| {
            let l = &map.lanes[lid];
            let start_i = l.src;
            let int = &map.intersections[start_i];
            int.turns_to(lid).map(|(tid, dir)| match dir {
                TraverseDirection::Forward => tid.src,
                TraverseDirection::Backward => tid.dst,
            })
        }) {
            let Some(ents) = self.lanes.get(&lane) else {
                continue;
            };
            for ent in ents {
                if self.reserved_by.contains(ent) {
                    continue;
                }
                let pos = self.positions.get(ent).unwrap();
                let dist = -pos.dist_along; // since dist_along is from start to end, a good dist_along is one that is big
                if lane == target_lane && pos.dist_along > start_along {
                    continue;
                }
                if dist < best_dist {
                    best_dist = dist;
                    best_ent = Some(*ent);
                }
            }
            if best_ent.is_some() {
                break;
            }
        }

        best_ent
    }
}

pub fn dispatch_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("map_dynamic::dispatch");

    let mut dispatcher = resources.write::<Dispatcher>();
    let map = resources.read::<Map>();
    dispatcher.update(&map, world);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{LanePatternBuilder, MapProject, ProjectKind};
    use common::rand::rand2;

    fn mk_ent(id: u64) -> DispatchID {
        DispatchID::FreightTrain(TrainID::from(KeyData::from_ffi(id)))
    }

    #[test]
    fn dispatch_one_register_one_works() {
        let mut disp = DispatchOne::new(LaneKind::Rail);
        let mut map = Map::default();

        let (_, r) = map
            .make_connection(
                MapProject::ground(Vec3::ZERO),
                MapProject::ground(Vec3::x(100.0)),
                None,
                &LanePatternBuilder::new().rail(true).build(),
            )
            .unwrap();

        let lanes: Vec<LaneID> = map.roads[r].lanes_iter().map(|(id, _)| id).collect();

        // first insert
        let ent = mk_ent(1 << 32);
        disp.register(ent, &map, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(disp.positions.len(), 1);
        assert_eq!(disp.lanes.len(), 1);
        assert_eq!(disp.lanes.values().next().unwrap()[0], ent);
        assert!(lanes.contains(disp.lanes.keys().next().unwrap()));

        // second insert in same lane
        let ent2 = mk_ent((1 << 32) + 1);
        disp.register(ent2, &map, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(disp.positions.len(), 2);
        assert_eq!(disp.lanes.len(), 1);
        assert_eq!(disp.lanes.values().next().unwrap(), &vec![ent, ent2]);

        // insert in another lane
        let ent3 = mk_ent((1 << 32) + 2);
        disp.register(ent3, &map, Vec3::new(100.0, 10.0, 0.0));
        assert_eq!(disp.positions.len(), 3);
        assert_eq!(disp.lanes.len(), 2);
        let mut v = disp.lanes.values();
        assert_eq!(v.next().unwrap(), &vec![ent, ent2]);
        assert_eq!(v.next().unwrap(), &vec![ent3]);

        // unregister
        disp.unregister(ent);
        assert_eq!(disp.positions.len(), 2);
        assert_eq!(disp.lanes.len(), 2);
        let mut v = disp.lanes.values();
        assert_eq!(v.next().unwrap(), &vec![ent2]);
        assert_eq!(v.next().unwrap(), &vec![ent3]);

        // unregister again
        disp.unregister(ent2);
        assert_eq!(disp.positions.len(), 1);
        assert_eq!(disp.lanes.len(), 2);
        let mut v = disp.lanes.values();
        assert_eq!(v.next().unwrap(), &vec![]);
        assert_eq!(v.next().unwrap(), &vec![ent3]);

        // ent3 moves from a lane to another
        disp.register(ent3, &map, Vec3::new(100.0, -1.0, 0.0));
        let mut v = disp.lanes.values();
        assert_eq!(v.next().unwrap(), &vec![ent3]);
        assert_eq!(v.next().unwrap(), &vec![]);

        // ent3 doesn't change lane because it's close to the old one
        disp.register(ent3, &map, Vec3::new(100.0, 1.0, 0.0));
        let mut v = disp.lanes.values();
        assert_eq!(v.next().unwrap(), &vec![ent3]);
        assert_eq!(v.next().unwrap(), &vec![]);
    }

    #[test]
    fn query_same_lane_works() {
        let mut d = Dispatcher::default();
        let mut map = Map::default();

        let (_, r) = map
            .make_connection(
                MapProject::ground(Vec3::ZERO),
                MapProject::ground(Vec3::x(100.0)),
                None,
                &LanePatternBuilder::new().one_way(true).rail(true).build(),
            )
            .unwrap();

        let (lid, _) = map.roads[r].lanes_iter().next().unwrap();

        let mut register = |id: DispatchID, pos: f32| {
            d.dispatches
                .entry(DispatchKind::FreightTrain)
                .or_insert(DispatchOne::new(DispatchKind::FreightTrain.lane_kind()))
                .register(id, &map, Vec3::x(pos))
        };

        let ent0 = mk_ent(1 << 32);
        let ent1 = mk_ent((1 << 32) + 1);
        let ent2 = mk_ent((1 << 32) + 2);

        register(ent0, 0.0);
        register(ent1, 10.0);
        register(ent2, 100.0);

        assert_eq!(
            d.query(
                &map,
                DispatchKind::FreightTrain,
                DispatchQueryTarget::Pos(Vec3::x(70.0)),
            ),
            Some(ent1)
        );
        d.dispatches[&DispatchKind::FreightTrain]
            .reserved_by
            .contains(&mk_ent((1 << 32) + 1));

        assert_eq!(
            d.query(
                &map,
                DispatchKind::FreightTrain,
                DispatchQueryTarget::Pos(Vec3::x(50.0)),
            ),
            Some(ent0)
        );
        d.dispatches[&DispatchKind::FreightTrain]
            .reserved_by
            .contains(&mk_ent(1 << 32));

        assert!(d
            .query(
                &map,
                DispatchKind::FreightTrain,
                DispatchQueryTarget::Pos(Vec3::x(50.0)),
            )
            .is_none());

        assert_eq!(
            d.query(
                &map,
                DispatchKind::FreightTrain,
                DispatchQueryTarget::Lane(lid),
            ),
            Some(ent2)
        );
    }

    #[test]
    fn query_two_lanes_bfs() {
        let mut d = Dispatcher::default();
        let mut map = Map::default();

        let (i, _) = map
            .make_connection(
                MapProject::ground(Vec3::ZERO),
                MapProject::ground(Vec3::x(100.0)),
                None,
                &LanePatternBuilder::new().one_way(true).rail(true).build(),
            )
            .unwrap();

        let (_, r2) = map
            .make_connection(
                MapProject {
                    kind: ProjectKind::Intersection(i),
                    pos: Vec3::x(100.0),
                },
                MapProject::ground(Vec3::new(200.0, 50.0, 0.0)),
                None,
                &LanePatternBuilder::new().one_way(true).rail(true).build(),
            )
            .unwrap();

        // unrelated
        map.make_connection(
            MapProject::ground(Vec3::new(0.0, 10.0, 0.0)),
            MapProject::ground(Vec3::new(100.0, 10.0, 0.0)),
            None,
            &LanePatternBuilder::new().one_way(true).rail(true).build(),
        )
        .unwrap();

        let (lid, _) = map.roads[r2].lanes_iter().next().unwrap();

        let mut register = |id: DispatchID, pos: f32| {
            d.dispatches
                .entry(DispatchKind::FreightTrain)
                .or_insert(DispatchOne::new(DispatchKind::FreightTrain.lane_kind()))
                .register(id, &map, Vec3::x(pos))
        };

        let ent0 = mk_ent(1 << 32);
        let ent1 = mk_ent((1 << 32) + 1);
        let ent2 = mk_ent((1 << 32) + 2);

        register(ent0, 0.0);
        register(ent1, 10.0);
        register(ent2, 200.0);

        assert_eq!(
            d.query(
                &map,
                DispatchKind::FreightTrain,
                DispatchQueryTarget::Pos(Vec3::x(70.0)),
            ),
            Some(ent1)
        );
        d.dispatches[&DispatchKind::FreightTrain]
            .reserved_by
            .contains(&mk_ent((1 << 32) + 1));

        assert_eq!(
            d.query(
                &map,
                DispatchKind::FreightTrain,
                DispatchQueryTarget::Pos(Vec3::x(50.0)),
            ),
            Some(ent0)
        );
        d.dispatches[&DispatchKind::FreightTrain]
            .reserved_by
            .contains(&mk_ent(1 << 32));

        assert!(d
            .query(
                &map,
                DispatchKind::FreightTrain,
                DispatchQueryTarget::Pos(Vec3::x(50.0)),
            )
            .is_none());

        assert_eq!(
            d.query(
                &map,
                DispatchKind::FreightTrain,
                DispatchQueryTarget::Lane(lid),
            ),
            Some(ent2)
        );
    }

    use crate::map::procgen::load_parismap;
    use easybench::bench;
    use slotmapd::KeyData;

    #[test]
    fn bench_query() {
        /* if 1 == 1 {
            return;
        }*/

        let mut m = Map::default();
        load_parismap(&mut m);

        let mut minx = f32::MAX;
        let mut maxx = f32::MIN;
        let mut miny = f32::MAX;
        let mut maxy = f32::MIN;
        for pos in m.intersections.iter().map(|i| i.1.pos) {
            minx = minx.min(pos.x);
            maxx = maxx.max(pos.x);
            miny = miny.min(pos.y);
            maxy = maxy.max(pos.y);
        }
        let w = maxx - minx;
        let h = maxy - miny;

        let mut start = DispatchOne::new(LaneKind::Driving);
        let mut i = 0;
        println!(
            "query empty: {}",
            bench(|| {
                i += 1;
                start.query(
                    &m,
                    DispatchKind::SmallTruck,
                    DispatchQueryTarget::Pos(Vec3::new(
                        minx + w * rand2(i as f32, 12.0),
                        miny + h * rand2(i as f32, 11.0),
                        0.0,
                    )),
                )
            })
        );

        for i in 0..100 {
            start.register(
                mk_ent((1 << 32) + i),
                &m,
                Vec3::new(
                    minx + w * rand2(i as f32, 2.0),
                    miny + h * rand2(i as f32, 1.0),
                    0.0,
                ),
            );
        }

        let mut i = 0;
        println!(
            "query 100: {}",
            bench(|| {
                i += 1;
                start.query(
                    &m,
                    DispatchKind::SmallTruck,
                    DispatchQueryTarget::Pos(Vec3::new(
                        minx + w * rand2(i as f32, 12.0),
                        miny + h * rand2(i as f32, 11.0),
                        0.0,
                    )),
                )
            })
        );

        for i in 100..1000 {
            start.register(
                mk_ent((1 << 32) + i),
                &m,
                Vec3::new(
                    minx + w * rand2(i as f32, 2.0),
                    miny + h * rand2(i as f32, 1.0),
                    0.0,
                ),
            );
        }

        let mut i = 0;
        println!(
            "query 1000: {}",
            bench(|| {
                i += 1;
                start.query(
                    &m,
                    DispatchKind::SmallTruck,
                    DispatchQueryTarget::Pos(Vec3::new(
                        minx + w * rand2(i as f32, 12.0),
                        miny + h * rand2(i as f32, 11.0),
                        0.0,
                    )),
                )
            })
        );

        for i in 1000..10000 {
            start.register(
                mk_ent((1 << 32) + i),
                &m,
                Vec3::new(
                    minx + w * rand2(i as f32, 2.0),
                    miny + h * rand2(i as f32, 1.0),
                    0.0,
                ),
            );
        }

        let mut i = 0;
        println!(
            "query 10000: {}",
            bench(|| {
                i += 1;
                start.query(
                    &m,
                    DispatchKind::SmallTruck,
                    DispatchQueryTarget::Pos(Vec3::new(
                        minx + w * rand2(i as f32, 12.0),
                        miny + h * rand2(i as f32, 11.0),
                        0.0,
                    )),
                )
            })
        );
    }
}
