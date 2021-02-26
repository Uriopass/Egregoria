use dashmap::DashMap;
use geom::Vec2;
use map_model::{LaneKind, Map, ParkingSpotID};
use serde::{Deserialize, Serialize};

register_resource!(ParkingManagement, "pmanagement");
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ParkingManagement {
    reserved_spots: DashMap<ParkingSpotID, ()>,
}

impl ParkingManagement {
    pub fn free(&self, spot: ParkingSpotID) {
        if self.reserved_spots.remove(&spot).is_none() {
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

                let plane = unwrap_or!(parent.parking_next_to(lane), continue);

                for spot in map.parking.closest_spots(plane, near) {
                    if self.reserved_spots.insert(spot, ()).is_none() {
                        return Some(spot);
                    }
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
