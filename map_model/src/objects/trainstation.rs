use crate::{IntersectionID, RoadID, TrainStations};
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
        lefti: IntersectionID,
        righti: IntersectionID,
        track: RoadID,
    ) -> TrainStationID {
        trainstations.insert(Self {
            left: lefti,
            right: righti,
            track,
        })
    }
}
