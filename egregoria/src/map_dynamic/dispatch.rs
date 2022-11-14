use crate::map::{LaneID, LaneKind, TraverseDirection};
use crate::utils::par_command_buffer::ComponentDrop;
use crate::Map;
use geom::{Transform, Vec3};
use hecs::{Entity, QueryBorrow, World};
use resources::Resources;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

/// How precise the dispatcher is. Caches dispatchable entities's positions and relation to map but only in precision circle.
/// So if a dispatchable entity moves less than the precision, nothing will be updated.
const PRECISION_RADIUS: f32 = 5.0;
const PRECISION_RADIUS_2: f32 = PRECISION_RADIUS * PRECISION_RADIUS;

/// Dispatcher is used to query for the closest networked entity matching a condition
/// For example:
/// - A rail fret station will query for the closest train to it that is not already used by another station
/// - A factory will query for a truck to deliver goods
/// - A hospital will query for the closest injured person
#[derive(Default, Serialize, Deserialize)]
pub struct Dispatcher {
    dispatches: BTreeMap<DispatchKind, DispatchOne>,
}

/// Dispatcher specialized to one kind
#[derive(Serialize, Deserialize)]
struct DispatchOne {
    positions: BTreeMap<Entity, DispatchPosition>,
    lanes: BTreeMap<LaneID, Vec<Entity>>,
    reserved_by: BTreeMap<Entity, Entity>,
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
#[derive(Serialize, Deserialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DispatchKind {
    FretTrain,
    SmallTruck,
}

impl DispatchKind {
    pub fn lane_kind(self) -> LaneKind {
        match self {
            DispatchKind::FretTrain => LaneKind::Rail,
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
    /// Update updates the dispatcher cache about the dispatachable entities to know where they are relative
    /// to the map, so that queries can be answered quickly
    pub fn update(
        &mut self,
        map: &Map,
        world: &World,
        mut query: QueryBorrow<(&Transform, &DispatchKind)>,
    ) {
        let mut disp: &mut DispatchOne = self
            .dispatches
            .entry(DispatchKind::FretTrain)
            .or_insert_with(|| DispatchOne::new(DispatchKind::FretTrain.lane_kind()));
        let mut last_kind: DispatchKind = DispatchKind::FretTrain;
        for (ent, (trans, kind)) in query.iter() {
            if last_kind != *kind {
                disp = self
                    .dispatches
                    .entry(*kind)
                    .or_insert_with(|| DispatchOne::new(kind.lane_kind()));
                last_kind = *kind;
            }

            if let Entry::Occupied(o) = disp.reserved_by.entry(ent) {
                if !world.contains(*o.get()) {
                    o.remove();
                } else {
                    return;
                }
            }
            disp.register(ent, map, trans.position);
        }
    }

    /// free says that the entity is no longer used by the target
    /// For example if a train is no longer used by a station, it should be freed so that other stations can use it
    /// It should be re-added to the cache at the next update iteration
    pub fn free(&mut self, kind: DispatchKind, ent: Entity) {
        let Some(disp) = self.dispatches.get_mut(&kind) else { return };
        disp.reserved_by.remove(&ent);
    }

    /// query reserves an entity (if it is found) and returns it
    /// it takes `me` as an argument so that if `me` is killed, the reservation is cancelled
    /// If no entity is found, returns None
    pub fn query(
        &mut self,
        map: &Map,
        me: Entity,
        kind: DispatchKind,
        target: DispatchQueryTarget,
    ) -> Option<Entity> {
        let disp = self.dispatches.get_mut(&kind)?;

        let mut start_along = f32::MAX;

        let target_lane = match target {
            DispatchQueryTarget::Pos(pos) => {
                let lid = map.nearest_lane(pos, kind.lane_kind(), Some(50.0))?;
                let lane = map.lanes().get(lid)?;
                let proj = lane.points.project(pos);
                start_along = lane.points.length_at_proj(proj);
                lid
            }
            DispatchQueryTarget::Lane(lane) => {
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
            let Some(ents) = disp.lanes.get(&lane) else { continue };
            for ent in ents {
                let pos = disp.positions.get(ent).unwrap();
                let dist = -pos.dist_along; // since dist_along is from start to end, a good dist_along is one that is big
                if lane == target_lane {
                    if pos.dist_along > start_along {
                        continue;
                    }
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

        let Some(ent) = best_ent else { return None };

        disp.reserve(ent, me);

        Some(ent)
    }
}

impl DispatchOne {
    fn new(lanekind: LaneKind) -> Self {
        Self {
            positions: BTreeMap::new(),
            lanes: BTreeMap::new(),
            reserved_by: BTreeMap::new(),
            lanekind,
        }
    }

    fn register(&mut self, id: Entity, map: &Map, pos: Vec3) {
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

    fn reserve(&mut self, id: Entity, me: Entity) {
        self.reserved_by.insert(id, me);
        let Some(pos) = self.positions.remove(&id) else {
            log::error!("Dispatcher: trying to reserve an entity that is not in the cache");
            return;
        };
        self.lanes.get_mut(&pos.lane).unwrap().retain(|e| *e != id);
    }

    pub fn unregister(&mut self, id: Entity) {
        self.reserved_by.remove(&id);
        let Some(pos) = self.positions.remove(&id) else { return };
        self.lanes.get_mut(&pos.lane).unwrap().retain(|e| *e != id);
    }
}

pub fn dispatch_system(world: &mut World, resources: &mut Resources) {
    let mut dispatcher = resources.get_mut::<Dispatcher>().unwrap();
    let map = resources.get::<Map>().unwrap();
    dispatcher.update(&map, world, world.query());
}

impl ComponentDrop for DispatchKind {
    fn drop(&mut self, goria: &mut Resources, ent: Entity) {
        let Ok(mut dispatcher) = goria.get_mut::<Dispatcher>() else { return };
        let Some(one) = dispatcher.dispatches.get_mut(self) else { return };
        one.unregister(ent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{MapProject, ProjectKind};
    use crate::LanePatternBuilder;
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
        let ent = Entity::from_bits(1 << 32).unwrap();
        disp.register(ent, &map, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(disp.positions.len(), 1);
        assert_eq!(disp.lanes.len(), 1);
        assert_eq!(disp.lanes.values().next().unwrap()[0], ent);
        assert!(lanes.contains(disp.lanes.keys().next().unwrap()));

        // second insert in same lane
        let ent2 = Entity::from_bits(1 << 32 + 1).unwrap();
        disp.register(ent2, &map, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(disp.positions.len(), 2);
        assert_eq!(disp.lanes.len(), 1);
        assert_eq!(disp.lanes.values().next().unwrap(), &vec![ent, ent2]);

        // insert in another lane
        let ent3 = Entity::from_bits(1 << 32 + 2).unwrap();
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

        let mut register = |id: Entity, pos: f32| {
            d.dispatches
                .entry(DispatchKind::FretTrain)
                .or_insert(DispatchOne::new(DispatchKind::FretTrain.lane_kind()))
                .register(id, &map, Vec3::x(pos))
        };

        let ent0 = Entity::from_bits(1 << 32).unwrap();
        let ent1 = Entity::from_bits((1 << 32) + 1).unwrap();
        let ent2 = Entity::from_bits((1 << 32) + 2).unwrap();
        let me = Entity::from_bits((1 << 32) + 3).unwrap();

        register(ent0, 0.0);
        register(ent1, 10.0);
        register(ent2, 100.0);

        assert_eq!(
            d.query(
                &map,
                me,
                DispatchKind::FretTrain,
                DispatchQueryTarget::Pos(Vec3::x(70.0)),
            ),
            Some(ent1)
        );
        d.dispatches[&DispatchKind::FretTrain]
            .reserved_by
            .contains_key(&Entity::from_bits((1 << 32) + 1).unwrap());

        assert_eq!(
            d.query(
                &map,
                me,
                DispatchKind::FretTrain,
                DispatchQueryTarget::Pos(Vec3::x(50.0)),
            ),
            Some(ent0)
        );
        d.dispatches[&DispatchKind::FretTrain]
            .reserved_by
            .contains_key(&Entity::from_bits(1 << 32).unwrap());

        assert!(d
            .query(
                &map,
                me,
                DispatchKind::FretTrain,
                DispatchQueryTarget::Pos(Vec3::x(50.0)),
            )
            .is_none());

        assert_eq!(
            d.query(
                &map,
                me,
                DispatchKind::FretTrain,
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
                    kind: ProjectKind::Inter(i),
                    pos: Vec3::x(100.0),
                },
                MapProject::ground(Vec3::x(200.0)),
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

        let mut register = |id: Entity, pos: f32| {
            d.dispatches
                .entry(DispatchKind::FretTrain)
                .or_insert(DispatchOne::new(DispatchKind::FretTrain.lane_kind()))
                .register(id, &map, Vec3::x(pos))
        };

        let ent0 = Entity::from_bits(1 << 32).unwrap();
        let ent1 = Entity::from_bits((1 << 32) + 1).unwrap();
        let ent2 = Entity::from_bits((1 << 32) + 2).unwrap();
        let me = Entity::from_bits((1 << 32) + 3).unwrap();

        register(ent0, 0.0);
        register(ent1, 10.0);
        register(ent2, 200.0);

        assert_eq!(
            d.query(
                &map,
                me,
                DispatchKind::FretTrain,
                DispatchQueryTarget::Pos(Vec3::x(70.0)),
            ),
            Some(ent1)
        );
        d.dispatches[&DispatchKind::FretTrain]
            .reserved_by
            .contains_key(&Entity::from_bits((1 << 32) + 1).unwrap());

        assert_eq!(
            d.query(
                &map,
                me,
                DispatchKind::FretTrain,
                DispatchQueryTarget::Pos(Vec3::x(50.0)),
            ),
            Some(ent0)
        );
        d.dispatches[&DispatchKind::FretTrain]
            .reserved_by
            .contains_key(&Entity::from_bits(1 << 32).unwrap());

        assert!(d
            .query(
                &map,
                me,
                DispatchKind::FretTrain,
                DispatchQueryTarget::Pos(Vec3::x(50.0)),
            )
            .is_none());

        assert_eq!(
            d.query(
                &map,
                me,
                DispatchKind::FretTrain,
                DispatchQueryTarget::Lane(lid),
            ),
            Some(ent2)
        );
    }
}
