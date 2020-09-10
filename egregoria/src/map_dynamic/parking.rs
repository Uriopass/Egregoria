use geom::Vec2;
use map_model::{LaneID, Map, ParkingSpotID};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Default, Serialize, Deserialize)]
pub struct ParkingManagement {
    reserved_spots: Mutex<HashSet<ParkingSpotID>>, // todo: use chashmap if it becomes a performance issue
}

impl ParkingManagement {
    pub fn free(&self, spot: ParkingSpotID) {
        assert!(
            self.reserved_spots.lock().unwrap().remove(&spot), // Unwrap ok: Mutex lives in the main thread
            "spot wasn't reserved"
        );
    }

    pub fn reserve_near(&self, lane: LaneID, near: Vec2, map: &Map) -> Option<ParkingSpotID> {
        let lane = map.lanes().get(lane)?;

        let mut reserved_spots = self.reserved_spots.lock().unwrap(); // Unwrap ok: Mutex lives in the main thread
        let depth = 3;

        let mut potential = vec![lane];
        let mut next = vec![];

        for _ in 0..depth {
            for lane in potential.drain(..) {
                let parent = unwrap_or!(map.roads().get(lane.parent), continue);

                let p = parent
                    .parking_next_to(lane)
                    .and_then(|x| map.parking.closest_available_spot(x, near, &reserved_spots));
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
