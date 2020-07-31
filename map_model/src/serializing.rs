use crate::{Houses, Intersections, Lanes, Map, ParkingSpots, Roads, SpatialMap};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SerializedMap {
    pub(crate) roads: Roads,
    pub(crate) intersections: Intersections,
    pub(crate) houses: Houses,
    pub(crate) lanes: Lanes,
    pub(crate) parking: ParkingSpots,
}

impl From<&Map> for SerializedMap {
    fn from(m: &Map) -> Self {
        let mut intersections = m.intersections.clone();
        for i in intersections.values_mut() {
            i.polygon.clear()
        }
        Self {
            roads: m.roads.clone(),
            intersections,
            houses: m.houses.clone(),
            lanes: m.lanes.clone(),
            parking: m.parking.clone(),
        }
    }
}

impl Into<Map> for SerializedMap {
    fn into(mut self) -> Map {
        let spatial_map = mk_spatial_map(&self);

        for inter in self.intersections.values_mut() {
            inter.update_polygon(&self.roads);
        }
        Map {
            roads: self.roads,
            lanes: self.lanes,
            intersections: self.intersections,
            houses: self.houses,
            spatial_map,
            parking: self.parking,
            dirty: false,
        }
    }
}

fn mk_spatial_map(m: &SerializedMap) -> SpatialMap {
    let mut sm = SpatialMap::default();
    for h in m.houses.values() {
        sm.insert_house(h);
    }
    for r in m.roads.values() {
        sm.insert_road(r);
    }
    sm
}
