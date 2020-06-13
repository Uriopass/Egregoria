use crate::geometry::Vec2;
use crate::map_model::{Lane, LaneID, LaneKind};
use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;

pub const PARKING_SPOT_LENGTH: f32 = 6.0;

#[derive(Serialize, Deserialize)]
pub struct ParkingSpot {
    pub pos: Vec2,
    pub orientation: Vec2,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ParkingSpots {
    spots: SecondaryMap<LaneID, Vec<ParkingSpot>>,
}

impl ParkingSpots {
    pub fn generate_spots(&mut self, lane: &Lane) {
        debug_assert!(matches!(lane.kind, LaneKind::Parking));
        let n_spots = (lane.length / PARKING_SPOT_LENGTH) as i32;
        let step = lane.length / n_spots as f32;

        let spots = match self.spots.get_mut(lane.id) {
            Some(x) => x,
            None => {
                self.spots.insert(lane.id, vec![]);
                &mut self.spots[lane.id]
            }
        };

        spots.clear();

        spots.extend(
            lane.points
                .points_dirs_along((0..n_spots).map(|x| (x as f32 + 0.5) * step))
                .map(|(pos, dir)| ParkingSpot {
                    pos,
                    orientation: dir,
                }),
        );
    }
}
