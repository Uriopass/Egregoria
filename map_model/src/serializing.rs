use crate::procgen::Trees;
use crate::{Buildings, Intersections, Lanes, Lots, Map, ParkingSpots, Roads, SpatialMap};
use geom::Shape;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct SerializedMap {
    pub roads: Roads,
    pub intersections: Intersections,
    pub buildings: Buildings,
    pub lanes: Lanes,
    pub parking: ParkingSpots,
    pub lots: Lots,
    pub trees: Trees,
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
            buildings: m.buildings.clone(),
            lanes: m.lanes.clone(),
            parking: m.parking.clone(),
            lots: m.lots.clone(),
            trees: m.trees.clone(),
        }
    }
}

impl From<SerializedMap> for Map {
    fn from(mut sel: SerializedMap) -> Self {
        for inter in sel.intersections.values_mut() {
            inter.update_polygon(&sel.roads);
        }

        let spatial_map = mk_spatial_map(&sel);
        Map {
            roads: sel.roads,
            lanes: sel.lanes,
            intersections: sel.intersections,
            buildings: sel.buildings,
            spatial_map,
            lots: sel.lots,
            parking: sel.parking,
            trees: sel.trees,
            dirt_id: 1,
        }
    }
}

fn mk_spatial_map(m: &SerializedMap) -> SpatialMap {
    let mut sm = SpatialMap::default();
    for h in m.buildings.values() {
        sm.insert(h.id, h.mesh.bbox());
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

impl Serialize for Map {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        SerializedMap::from(self).serialize(serializer)
    }
}
