use serde::{Deserialize, Serialize};

use crate::map::{
    BuildingID, Buildings, ElectricityCache, Environment, Intersections, Lanes, Lots, Map,
    ParkingSpots, Roads, SpatialMap,
};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct SerializedMap {
    pub roads: Roads,
    pub intersections: Intersections,
    pub buildings: Buildings,
    pub lanes: Lanes,
    pub parking: ParkingSpots,
    pub lots: Lots,
    pub environment: Environment,
    pub external_train_stations: Vec<BuildingID>,
}

impl From<&Map> for SerializedMap {
    fn from(m: &Map) -> Self {
        Self {
            roads: m.roads.clone(),
            intersections: m.intersections.clone(),
            buildings: m.buildings.clone(),
            lanes: m.lanes.clone(),
            parking: m.parking.clone(),
            lots: m.lots.clone(),
            environment: m.environment.clone(),
            external_train_stations: m.external_train_stations.clone(),
        }
    }
}

impl From<SerializedMap> for Map {
    fn from(sel: SerializedMap) -> Self {
        let spatial_map = mk_spatial_map(&sel);
        let mut m = Map {
            roads: sel.roads,
            lanes: sel.lanes,
            intersections: sel.intersections,
            buildings: sel.buildings,
            spatial_map,
            lots: sel.lots,
            parking: sel.parking,
            environment: sel.environment,
            external_train_stations: sel.external_train_stations,
            ..Self::empty()
        };
        m.electricity = ElectricityCache::build(&m);
        m
    }
}

fn mk_spatial_map(m: &SerializedMap) -> SpatialMap {
    let mut sm = SpatialMap::default();
    for b in m.buildings.values() {
        sm.insert(b);
    }
    for r in m.roads.values() {
        sm.insert(r);
    }
    for i in m.intersections.values() {
        sm.insert(i);
    }
    for l in m.lots.values() {
        sm.insert(l);
    }
    sm
}
