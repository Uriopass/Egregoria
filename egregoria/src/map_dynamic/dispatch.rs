use crate::map::{LaneID, LaneKind};
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
#[derive(Default, Serialize, Deserialize)]
struct DispatchOne {
    positions: BTreeMap<Entity, DispatchPosition>,
    lanes: BTreeMap<LaneID, Vec<Entity>>,
    reserved_by: BTreeMap<Entity, Entity>,
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
        let mut disp: &mut DispatchOne =
            self.dispatches.entry(DispatchKind::FretTrain).or_default();
        let mut last_kind: DispatchKind = DispatchKind::FretTrain;
        for (ent, (trans, kind)) in query.iter() {
            if last_kind != *kind {
                disp = self.dispatches.entry(*kind).or_default();
                last_kind = *kind;
            }

            if let Entry::Occupied(o) = disp.reserved_by.entry(ent) {
                if !world.contains(*o.get()) {
                    o.remove();
                } else {
                    return;
                }
            }
            disp.register(ent, *kind, map, trans.position);
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

        let mut start_along = 0.0;

        let target_lane = match target {
            DispatchQueryTarget::Pos(pos) => {
                let lid = map.nearest_lane(pos, kind.lane_kind(), Some(50.0))?;
                let lane = map.lanes().get(lid)?;
                let proj = lane.points.project(pos);
                start_along = -lane.points.length_at_proj(proj);
                lid
            }
            DispatchQueryTarget::Lane(lane) => lane,
        };

        let mut best_dist = f32::MAX;
        let mut best_ent = None;

        let mut queue = vec![];
        // do a backward breadth first search, looking for lanes with matching entities

        queue.push(target_lane);
        while let Some(lane) = queue.pop() {
            if let Some(ents) = disp.lanes.get(&lane) {
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
            }
            /*
            for l in map.lanes.get(lane)?.iter() {
                queue.push(*l);
            }*/
        }

        let Some(ent) = best_ent else { return None };

        disp.reserve(ent, me);

        Some(ent)
    }
}

impl DispatchOne {
    fn register(&mut self, id: Entity, kind: DispatchKind, map: &Map, pos: Vec3) {
        let ent = self.positions.entry(id);

        let find_lane = || map.nearest_lane(pos, kind.lane_kind(), Some(50.0));

        match ent {
            Entry::Vacant(v) => {
                let Some(n) = find_lane() else { return };
                self.lanes.entry(n).or_default().push(id);
                v.insert(DispatchPosition {
                    lane: n,
                    pos,
                    dist_along: 0.0,
                });
            }
            Entry::Occupied(mut o) => {
                let dp = o.get();

                if dp.pos.distance2(pos) < PRECISION_RADIUS_2 {
                    return;
                }

                if let Some(l) = map.lanes().get(dp.lane) {
                    if l.points.project_dist2(pos) < PRECISION_RADIUS_2 {
                        return;
                    }
                }

                let Some(n) = find_lane() else { return };
                self.lanes.get_mut(&dp.lane).unwrap().retain(|e| *e != id);
                self.lanes.entry(n).or_default().push(id);
                o.insert(DispatchPosition {
                    lane: n,
                    pos,
                    dist_along: 0.0,
                });
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
    use crate::map::MapProject;
    use crate::LanePatternBuilder;
    #[test]
    fn dispatch_one_register_one_works() {
        let mut disp = DispatchOne::default();
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
        disp.register(ent, DispatchKind::FretTrain, &map, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(disp.positions.len(), 1);
        assert_eq!(disp.lanes.len(), 1);
        assert_eq!(disp.lanes.values().next().unwrap()[0], ent);
        assert!(lanes.contains(disp.lanes.keys().next().unwrap()));

        // second insert in same lane
        let ent2 = Entity::from_bits(1 << 32 + 1).unwrap();
        disp.register(
            ent2,
            DispatchKind::FretTrain,
            &map,
            Vec3::new(0.0, 0.0, 0.0),
        );
        assert_eq!(disp.positions.len(), 2);
        assert_eq!(disp.lanes.len(), 1);
        assert_eq!(disp.lanes.values().next().unwrap(), &vec![ent, ent2]);

        // insert in another lane
        let ent3 = Entity::from_bits(1 << 32 + 2).unwrap();
        disp.register(
            ent3,
            DispatchKind::FretTrain,
            &map,
            Vec3::new(100.0, 10.0, 0.0),
        );
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
        disp.register(
            ent3,
            DispatchKind::FretTrain,
            &map,
            Vec3::new(100.0, -1.0, 0.0),
        );
        let mut v = disp.lanes.values();
        assert_eq!(v.next().unwrap(), &vec![ent3]);
        assert_eq!(v.next().unwrap(), &vec![]);

        // ent3 doesn't change lane because it's close to the old one
        disp.register(
            ent3,
            DispatchKind::FretTrain,
            &map,
            Vec3::new(100.0, 1.0, 0.0),
        );
        let mut v = disp.lanes.values();
        assert_eq!(v.next().unwrap(), &vec![ent3]);
        assert_eq!(v.next().unwrap(), &vec![]);
    }
}
