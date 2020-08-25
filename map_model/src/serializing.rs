use crate::{Houses, Intersections, Lanes, Lots, Map, ParkingSpots, Roads, SpatialMap, Workplaces};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct SerializedMap {
    pub(crate) roads: Roads,
    pub(crate) intersections: Intersections,
    pub(crate) houses: Houses,
    pub(crate) lanes: Lanes,
    pub(crate) parking: ParkingSpots,
    pub(crate) lots: Lots,
    pub(crate) workplace: Workplaces,
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
            lots: m.lots.clone(),
            workplace: m.workplaces.clone(),
        }
    }
}

impl Into<Map> for SerializedMap {
    fn into(mut self) -> Map {
        for inter in self.intersections.values_mut() {
            inter.update_polygon(&self.roads);
        }

        let spatial_map = mk_spatial_map(&self);
        Map {
            roads: self.roads,
            lanes: self.lanes,
            intersections: self.intersections,
            houses: self.houses,
            spatial_map,
            lots: self.lots,
            parking: self.parking,
            workplaces: self.workplace,
            dirty: false,
        }
    }
}

fn mk_spatial_map(m: &SerializedMap) -> SpatialMap {
    let mut sm = SpatialMap::default();
    for w in m.workplace.values() {
        //        sm.insert(w.id, w.exterior.bbox());
    }
    for h in m.houses.values() {
        sm.insert(h.id, h.exterior.bbox());
    }
    for r in m.roads.values() {
        sm.insert(r.id, r.bbox());
    }
    for i in m.intersections.values() {
        sm.insert(i.id, i.polygon.bbox());
    }
    for l in m.lots.values() {
        sm.insert(l.id, l.shape.bbox());
    }
    sm
}
