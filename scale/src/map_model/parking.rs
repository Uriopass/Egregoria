use crate::geometry::Vec2;
use crate::map_model::{Lane, LaneID, LaneKind, Map};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use std::collections::HashSet;
use std::sync::Mutex;

new_key_type! {
    pub struct ParkingSpotID;
}

pub const PARKING_SPOT_LENGTH: f32 = 6.0;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ParkingSpot {
    pub pos: Vec2,
    pub orientation: Vec2,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ParkingSpots {
    spots: SlotMap<ParkingSpotID, ParkingSpot>,
    lane_spots: SecondaryMap<LaneID, Vec<ParkingSpotID>>,
    reserved_spots: Mutex<HashSet<ParkingSpotID>>, // todo: use chashmap if it becomes a performance issue
}

impl ParkingSpots {
    pub fn get(&self, spot: ParkingSpotID) -> Option<&ParkingSpot> {
        self.spots.get(spot)
    }

    pub fn generate_spots(&mut self, lane: &Lane) {
        debug_assert!(matches!(lane.kind, LaneKind::Parking));

        let lane_spots = match self.lane_spots.get_mut(lane.id) {
            Some(x) => x,
            None => {
                self.lane_spots.insert(lane.id, vec![]);
                &mut self.lane_spots[lane.id]
            }
        };

        let n_spots = (lane.length / PARKING_SPOT_LENGTH) as i32;
        let step = lane.length / n_spots as f32;

        for spot in lane_spots.drain(..) {
            self.spots.remove(spot);
        }

        let spots = &mut self.spots;
        lane_spots.extend(
            lane.points
                .points_dirs_along((0..n_spots).map(|x| (x as f32 + 0.5) * step))
                .map(|(pos, dir)| {
                    spots.insert(ParkingSpot {
                        pos,
                        orientation: dir,
                    })
                }),
        );
    }

    pub fn closest_available_spot(
        &self,
        lane: LaneID,
        near: Vec2,
        reserved_spots: &HashSet<ParkingSpotID>,
    ) -> Option<ParkingSpotID> {
        debug_assert!(self.lane_spots.contains_key(lane));
        let spots = &self.spots;
        self.lane_spots
            .get(lane)?
            .iter()
            .filter(|&p| !reserved_spots.contains(p))
            .min_by_key(|&&id| OrderedFloat(spots[id].pos.distance2(near)))
            .copied()
    }

    pub fn unreserve(&self, spot: ParkingSpotID) {
        self.reserved_spots.lock().unwrap().remove(&spot);
    }

    pub fn reserve_near(&self, lane: LaneID, near: Vec2, map: &Map) -> Option<ParkingSpotID> {
        let lane = map.lanes().get(lane)?;

        let mut reserved_spots = self.reserved_spots.lock().unwrap();
        let depth = 3;

        let mut potential = vec![lane];
        let mut next = vec![];

        for _ in 0..depth {
            for lane in potential.drain(..) {
                let parent = unwrap_or!(map.roads().get(lane.parent), continue);

                let p = parent
                    .parking_next_to(lane)
                    .and_then(|x| self.closest_available_spot(x, near, &reserved_spots));
                if let Some(spot) = p {
                    reserved_spots.insert(spot);
                    return Some(spot);
                }

                next.extend(
                    map.intersections()[lane.dst]
                        .turns_from(lane.id)
                        .map(|(turn, _)| &map.lanes()[turn.dst]),
                )
            }
            std::mem::swap(&mut potential, &mut next);
        }
        None
    }
}
