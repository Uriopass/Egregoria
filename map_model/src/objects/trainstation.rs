use crate::{
    Intersection, IntersectionID, Intersections, LaneKind, LanePattern, Lanes, ParkingSpots, Road,
    RoadID, RoadSegmentKind, Roads, SpatialMap, TrainStations,
};
use geom::Vec3;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct TrainStationID;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrainStation {
    pub left: IntersectionID,
    pub right: IntersectionID,
    pub track: RoadID,
}

impl TrainStation {
    pub fn make(
        trainstations: &mut TrainStations,
        roads: &mut Roads,
        lanes: &mut Lanes,
        parking: &mut ParkingSpots,
        inters: &mut Intersections,
        spatial: &mut SpatialMap,
        left: Vec3,
        right: Vec3,
    ) -> TrainStationID {
        let lefti = Intersection::make(inters, spatial, left);
        let righti = Intersection::make(inters, spatial, right);
        let track = Road::make(
            &inters[lefti],
            &inters[righti],
            RoadSegmentKind::Straight,
            &LanePattern {
                lanes_forward: vec![(LaneKind::Rail, 30.0)],
                lanes_backward: vec![(LaneKind::Rail, 30.0)],
            },
            roads,
            lanes,
            parking,
            spatial,
        );

        trainstations.insert(Self {
            left: lefti,
            right: righti,
            track,
        })
    }
}
