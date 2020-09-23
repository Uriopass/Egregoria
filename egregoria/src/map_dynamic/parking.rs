use dashmap::DashMap;
use geom::Vec2;
use map_model::{LaneKind, Map, ParkingSpotID};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct ParkingManagement {
    reserved_spots: DashMap<ParkingSpotID, ()>,
}

impl ParkingManagement {
    pub fn free(&self, spot: ParkingSpotID) {
        if !self.reserved_spots.remove(&spot) {
            log::warn!("{:?} wasn't reserved", spot);
        }
    }

    pub fn reserve_near(&self, near: Vec2, map: &Map) -> Option<ParkingSpotID> {
        let lane = map.nearest_lane(near, LaneKind::Parking)?;
        let lane = map.lanes().get(lane)?;

        let depth = 3;

        let mut potential = vec![lane];
        let mut next = vec![];

        for _ in 0..depth {
            for lane in potential.drain(..) {
                let parent = unwrap_or!(map.roads().get(lane.parent), continue);

                let p = parent.parking_next_to(lane).and_then(|x| {
                    map.parking
                        .closest_available_spot(x, near, |p| self.reserved_spots.contains_key(p))
                });
                if let Some(spot) = p {
                    self.reserved_spots.insert(spot, ());
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
