use crate::map::{BuildingID, Buildings, IntersectionID, Intersections, Map, RoadID, Roads};
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

/// A network object is an object that can be connected to an electricity network
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NetworkObjectID {
    Building(BuildingID),
    Intersection(IntersectionID),
    Road(RoadID),
}

impl From<BuildingID> for NetworkObjectID {
    fn from(v: BuildingID) -> Self {
        Self::Building(v)
    }
}

impl From<IntersectionID> for NetworkObjectID {
    fn from(v: IntersectionID) -> Self {
        Self::Intersection(v)
    }
}

impl From<RoadID> for NetworkObjectID {
    fn from(v: RoadID) -> Self {
        Self::Road(v)
    }
}

/// The id of a network is the id of its lowest object. This is necessary to keep everything
/// deterministic even though we don't serialize the electricity cache
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ElectricityNetworkID(NetworkObjectID);

/// A network is a set of sources and sinks that is connected together
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ElectricityNetwork {
    pub id: ElectricityNetworkID,

    /// The sources/sinks of the network must be buildings. For efficient iteration,
    /// we store them separately from the road graph
    pub buildings: BTreeSet<BuildingID>,

    /// The objects of the networks
    pub objects: BTreeSet<NetworkObjectID>,
}

/// The electricity cache is a cache of all the electricity networks in the map
/// It maintains a mapping from network objects to network ids that are connected to each other
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct ElectricityCache {
    pub(crate) networks: BTreeMap<ElectricityNetworkID, ElectricityNetwork>,

    /// The network that each intersection is connected to
    pub(crate) ids: BTreeMap<NetworkObjectID, ElectricityNetworkID>,

    /// The memoized graph of electricity edges
    /// This is used to decouple the adding/removal of edges and actual removal in map
    /// Note that the ordering of the Vec isn't deterministic so shouldn't be used for anything
    pub(crate) graph: BTreeMap<NetworkObjectID, Vec<NetworkObjectID>>,
}

impl ElectricityCache {
    /// Add a new network object. Must be called before adding or removing edges.
    pub fn add_object(&mut self, object_id: impl Into<NetworkObjectID>) {
        let object_id = object_id.into();
        self.add_object_inner(object_id);
    }
    fn add_object_inner(&mut self, object_id: NetworkObjectID) {
        let e = self.graph.entry(object_id);

        match e {
            Entry::Vacant(v) => v.insert(Vec::new()),
            Entry::Occupied(_) => return,
        };

        let network_id = ElectricityNetworkID(object_id);
        let network = ElectricityNetwork {
            id: network_id,
            buildings: match object_id {
                NetworkObjectID::Building(b) => BTreeSet::from([b]),
                _ => BTreeSet::new(),
            },
            objects: BTreeSet::from([object_id]),
        };

        self.networks.insert(network_id, network);
        self.ids.insert(object_id, network_id);
    }

    /// Remove a network object.
    pub fn remove_object(&mut self, object_id: impl Into<NetworkObjectID>) {
        let object_id = object_id.into();
        self.remove_object_inner(object_id);
    }
    fn remove_object_inner(&mut self, object_id: NetworkObjectID) {
        // Even though we remove edges in "random order", it doesn't matter because
        // the network ids are defined by the smallest object in the network (which is deterministic)
        // We don't remove the edges directly so we can remove edge by edge to maintain coherency
        let Some(edges) = self.graph.get(&object_id).cloned() else {
            return;
        };
        for edge in edges.iter() {
            self.remove_edge_inner(&object_id, edge);
        }

        let network_id = self.ids.remove(&object_id).unwrap();
        let network = self.networks.remove(&network_id).unwrap();
        self.graph.remove(&object_id);

        debug_assert!(network.objects.len() == 1, "{:?}", object_id);
    }

    /// Add an edge between two network objects (symmetric)
    /// Must agree with the electricity_edge function
    pub fn add_edge(&mut self, src: impl Into<NetworkObjectID>, dst: impl Into<NetworkObjectID>) {
        let src = src.into();
        let dst = dst.into();
        self.add_edge_inner(&src, &dst);
    }
    fn add_edge_inner(&mut self, src: &NetworkObjectID, dst: &NetworkObjectID) {
        if self.graph.get(src).unwrap().contains(dst) {
            return;
        }

        self.graph.get_mut(src).unwrap().push(*dst);
        self.graph.get_mut(dst).unwrap().push(*src);

        let Some(src) = self.ids.get(src) else {
            log::error!("electricity add_edge src {:?} not found", src);
            return;
        };
        let Some(dst) = self.ids.get(dst) else {
            log::error!("electricity add_edge dst {:?} not found", dst);
            return;
        };
        self.merge(*src, *dst);
    }

    /// Remove an edge between two network objects (symmetric)
    /// Must agree with the electricity_edge function
    /// Must be called _after_ removing the edge from the map
    pub fn remove_edge(
        &mut self,
        src: impl Into<NetworkObjectID>,
        dst: impl Into<NetworkObjectID>,
    ) {
        let src = src.into();
        let dst = dst.into();
        self.remove_edge_inner(&src, &dst);
    }
    fn remove_edge_inner(&mut self, src: &NetworkObjectID, dst: &NetworkObjectID) {
        let Some(g_src) = self.graph.get(src) else {
            return;
        };
        if !g_src.contains(dst) {
            return;
        }
        // Even though we use retain for removal which ends up as O(degree²) the degree is never big
        self.graph.get_mut(src).unwrap().retain(|v| v != dst);
        self.graph.get_mut(dst).unwrap().retain(|v| v != src);

        let Some(src_net) = self.ids.get(src) else {
            log::error!("electricity remove_edge src {:?} not found", src);
            return;
        };
        let Some(dst_net) = self.ids.get(dst) else {
            log::error!("electricity remove_edge dst {:?} not found", dst);
            return;
        };
        debug_assert!(src_net == dst_net);
        if self.path_exists(*src, *dst) {
            return;
        }
        self.split(*src_net, *src, *dst);
    }

    fn path_exists(&self, src: NetworkObjectID, dst: NetworkObjectID) -> bool {
        for v in pathfinding::directed::bfs::bfs_reach(src, |n| self.edges(*n)) {
            if v == dst {
                return true;
            }
        }
        false
    }

    /// Gets the network id of a network object
    /// Note that network ids change all the time, so this should not be kept as state
    pub fn net_id(&self, object_id: impl Into<NetworkObjectID>) -> Option<ElectricityNetworkID> {
        self.ids.get(&object_id.into()).copied()
    }

    pub fn networks(&self) -> impl Iterator<Item = &ElectricityNetwork> {
        self.networks.values()
    }

    pub fn graph(&self) -> &BTreeMap<NetworkObjectID, Vec<NetworkObjectID>> {
        &self.graph
    }

    /// Build the electricity cache from a map. Should give the same result as the current cache in the map
    pub fn build(map: &Map) -> ElectricityCache {
        let mut e = ElectricityCache::default();

        for b in map.buildings.keys() {
            e.add_object(NetworkObjectID::Building(b));
        }

        for i in map.intersections.keys() {
            e.add_object(NetworkObjectID::Intersection(i));
        }

        for r in map.roads.keys() {
            e.add_object(NetworkObjectID::Road(r));
        }

        for n_id in common::iter::chain((
            map.buildings.keys().map(NetworkObjectID::Building),
            map.intersections.keys().map(NetworkObjectID::Intersection),
            map.roads.keys().map(NetworkObjectID::Road),
        )) {
            for neighbor in
                Self::map_electricity_edges(&map.roads, &map.buildings, &map.intersections, n_id)
            {
                e.add_edge(n_id, neighbor);
            }
        }

        e
    }

    /// Must agree with the map_eletricity_edges function in the end
    fn edges(&self, id: NetworkObjectID) -> impl Iterator<Item = NetworkObjectID> + '_ {
        self.graph.get(&id).unwrap().iter().copied()
    }

    /// Iterate over the edges of a network object
    ///
    /// Buildings -> 1 road
    /// Intersections -> n roads
    /// Roads -> 2 intersections + n buildings
    fn map_electricity_edges<'a>(
        roads: &'a Roads,
        buildings: &'a Buildings,
        intersections: &'a Intersections,
        obj: NetworkObjectID,
    ) -> impl Iterator<Item = NetworkObjectID> + 'a {
        use itertools::Either::{Left, Right};
        match obj {
            NetworkObjectID::Building(b) => {
                let Some(b) = buildings.get(b) else {
                    return Left(Left(std::iter::empty()));
                };
                let Some(r) = b.connected_road else {
                    return Left(Left(std::iter::empty()));
                };
                Left(Right(Some(NetworkObjectID::Road(r)).into_iter()))
            }
            NetworkObjectID::Intersection(i) => {
                let Some(i) = intersections.get(i) else {
                    return Left(Left(std::iter::empty()));
                };
                Right(Left(i.roads.iter().map(|v| NetworkObjectID::Road(*v))))
            }
            NetworkObjectID::Road(r) => {
                let Some(r) = roads.get(r) else {
                    return Left(Left(std::iter::empty()));
                };
                Right(Right(common::iter::chain((
                    Some(NetworkObjectID::Intersection(r.src)).into_iter(),
                    Some(NetworkObjectID::Intersection(r.dst)).into_iter(),
                    r.connected_buildings
                        .iter()
                        .map(|v| NetworkObjectID::Building(*v)),
                ))))
            }
        }
    }

    /// Merge two networks together, the smallest one is removed
    ///
    /// The strategy is:
    ///  - Merge objects/sources/sinks into the biggest existing network to avoid allocations
    ///  - Find out the network_id of the new network (smallest object within it)
    ///  - Update the network_id of all objects in the new network if needed
    ///
    fn merge(&mut self, mut a: ElectricityNetworkID, mut b: ElectricityNetworkID) {
        if a == b {
            return;
        }

        if self.networks[&a].objects.len() > self.networks[&b].objects.len() {
            std::mem::swap(&mut a, &mut b);
        }

        let src = self.networks.remove(&a).unwrap();
        let dst = self.networks.get(&b).unwrap();

        let new_id = ElectricityNetworkID(
            *dst.objects
                .first()
                .unwrap()
                .min(src.objects.first().unwrap()),
        );

        // We will need to relabel all objects in src in any case
        for id in src.objects.iter() {
            self.ids.insert(*id, new_id);
        }

        // network_id changed, we need to relabel dst objects too
        // and re-insert it into the networks btree
        if new_id != dst.id {
            for id in dst.objects.iter() {
                self.ids.insert(*id, new_id);
            }

            let dst_id = dst.id; // cannot inline cuz of borrow checker
            let mut dst = self.networks.remove(&dst_id).unwrap();
            dst.id = new_id;
            self.networks.insert(dst.id, dst);
        }

        // finally, merge src into dst
        let dst = self.networks.get_mut(&new_id).unwrap();
        dst.buildings.extend(src.buildings);
        dst.objects.extend(src.objects);
    }

    /// Split a network into two networks
    /// The two given ids are hints of elements in disjoint connected components
    fn split(
        &mut self,
        network_to_split_id: ElectricityNetworkID,
        id1: NetworkObjectID,
        id2: NetworkObjectID,
    ) {
        fn explore(
            cache: &ElectricityCache,
            visited: &mut BTreeSet<NetworkObjectID>,
            id: NetworkObjectID,
            early_stop: NetworkObjectID, // we can stop when we find the old network since we won't touch it
        ) -> bool {
            visited.insert(id);
            if early_stop == id {
                return true;
            }
            for neighbor in cache.edges(id) {
                if visited.contains(&neighbor) {
                    continue;
                }
                if explore(cache, visited, neighbor, early_stop) {
                    return true;
                }
            }
            false
        }

        let mut visited1 = BTreeSet::new();
        let mut visited2 = BTreeSet::new();

        explore(self, &mut visited1, id1, network_to_split_id.0);
        explore(self, &mut visited2, id2, network_to_split_id.0);

        let network_1_id = ElectricityNetworkID(*visited1.first().unwrap());
        let network_2_id = ElectricityNetworkID(*visited2.first().unwrap());

        debug_assert!(network_1_id != network_2_id, "path existed");

        fn apply_split(
            cache: &mut ElectricityCache,
            kept_network_id: ElectricityNetworkID,
            new_network_id: ElectricityNetworkID,
            new_network_objects: BTreeSet<NetworkObjectID>,
        ) {
            let kept_net = cache.networks.get_mut(&kept_network_id).unwrap();
            let mut new_buildings = BTreeSet::new();
            for v in new_network_objects.iter() {
                kept_net.objects.remove(v);
                if let NetworkObjectID::Building(b) = v {
                    kept_net.buildings.remove(b);
                    new_buildings.insert(*b);
                }
                cache.ids.insert(*v, new_network_id);
            }

            let new_network = ElectricityNetwork {
                id: new_network_id,
                buildings: new_buildings,
                objects: new_network_objects,
            };

            cache.networks.insert(new_network_id, new_network);
        }

        if network_to_split_id == network_1_id {
            apply_split(self, network_1_id, network_2_id, visited2)
        } else if network_to_split_id == network_2_id {
            apply_split(self, network_2_id, network_1_id, visited1)
        } else {
            debug_assert!(false, "network_id not found");
        }
    }
}

pub fn check_electricity_coherency(map: &Map) {
    let mut e_from_map = ElectricityCache::build(map);
    let mut e = map.electricity.clone();
    for v in e.graph.values_mut() {
        v.sort();
    }
    for v in e_from_map.graph.values_mut() {
        v.sort();
    }
    assert_eq!(e, e_from_map);
}

#[cfg(test)]
mod tests {
    use crate::map::{check_electricity_coherency, ElectricityCache};
    use crate::map::{BuildingKind, LanePatternBuilder, Map, MapProject, NetworkObjectID, RoadID};
    use common::logger::MyLog;
    use geom::{vec3, Vec2, OBB};
    use prototypes::BuildingGen;
    use slotmapd::KeyData;

    #[test]
    fn test_loop_removal() {
        MyLog::init();

        let mut e = ElectricityCache::default();

        let mk_ent = |i| NetworkObjectID::Road(RoadID::from(KeyData::from_ffi(i)));

        e.add_object(mk_ent(1));
        e.add_object(mk_ent(2));
        e.add_object(mk_ent(3));
        e.add_object(mk_ent(4));
        e.add_object(mk_ent(5));
        e.add_object(mk_ent(6));
        e.add_object(mk_ent(7));
        e.add_object(mk_ent(8));

        e.add_edge(mk_ent(1), mk_ent(2));
        e.add_edge(mk_ent(2), mk_ent(3));
        e.add_edge(mk_ent(3), mk_ent(4));
        e.add_edge(mk_ent(4), mk_ent(5));
        e.add_edge(mk_ent(5), mk_ent(6));
        e.add_edge(mk_ent(6), mk_ent(7));
        e.add_edge(mk_ent(7), mk_ent(8));
        e.add_edge(mk_ent(8), mk_ent(1));

        assert_eq!(e.networks.len(), 1);

        e.remove_edge(mk_ent(1), mk_ent(2));
        e.remove_edge(mk_ent(2), mk_ent(3));
        e.remove_object(mk_ent(2));

        assert_eq!(e.networks.len(), 1);
    }

    #[test]
    fn test_connectivity() {
        MyLog::init();
        let mut m = Map::empty();

        let (_, r) = m
            .make_connection(
                MapProject::ground(vec3(0.0, 0.0, 0.0)),
                MapProject::ground(vec3(1.0, 0.0, 0.0)),
                None,
                &LanePatternBuilder::new().build(),
            )
            .unwrap();

        let b = m
            .build_special_building(
                &OBB::ZERO,
                BuildingKind::ExternalTrading,
                BuildingGen::NoWalkway {
                    door_pos: Vec2::ZERO,
                },
                None,
                Some(r),
            )
            .unwrap();

        let mut e = ElectricityCache::build(&m);
        check_electricity_coherency(&m);

        assert_eq!(e.networks.len(), 1);
        assert_eq!(e.networks[&e.net_id(b).unwrap()].objects.len(), 4);
        assert_eq!(e.networks[&e.net_id(b).unwrap()].buildings.len(), 1);

        e.remove_edge(r, b);

        assert_eq!(e.networks.len(), 2);
        assert_eq!(e.networks[&e.net_id(b).unwrap()].buildings.len(), 1);
        assert_eq!(e.networks[&e.net_id(r).unwrap()].buildings.len(), 0);

        e.add_edge(r, b);

        m.remove_road(r);

        let e = &mut m.electricity;
        assert_eq!(e.networks.len(), 1);
        assert_eq!(e.networks[&e.net_id(b).unwrap()].buildings.len(), 1);
    }
}
