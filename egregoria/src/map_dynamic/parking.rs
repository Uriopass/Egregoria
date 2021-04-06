use common::{FastMap, PtrCmp};
use geom::Vec2;
use map_model::{LaneKind, Map, ParkingSpotID};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;

register_resource!(ParkingManagement, "pmanagement");
#[derive(Default, Serialize, Deserialize)]
pub struct ParkingManagement {
    reserved_spots: RwLock<FastMap<ParkingSpotID, ()>>,
}

impl ParkingManagement {
    pub fn free(&self, spot: ParkingSpotID) {
        if self.reserved_spots.write().unwrap().remove(&spot).is_none() {
            log::warn!("{:?} wasn't reserved", spot);
        }
    }

    pub fn is_free(&self, spot: ParkingSpotID) -> bool {
        self.reserved_spots.read().unwrap().contains_key(&spot)
    }

    pub fn reserve_near(&self, near: Vec2, map: &Map) -> Option<ParkingSpotID> {
        let lane = map.nearest_lane(near, LaneKind::Driving)?;
        let lane = map.lanes().get(lane)?;

        let depth = 7;

        let mut potential = HashSet::new();
        potential.insert(PtrCmp(lane));
        let mut next = HashSet::new();
        let intersections = map.intersections();
        let roads = map.roads();
        for _ in 0..depth {
            for lane in potential.drain() {
                let lane = lane.0;
                let parent = unwrap_or!(roads.get(lane.parent), continue);

                let plane = unwrap_or!(parent.parking_next_to(lane), continue);

                for spot in map.parking.closest_spots(plane, near) {
                    if self
                        .reserved_spots
                        .write()
                        .unwrap()
                        .insert(spot, ())
                        .is_none()
                    {
                        return Some(spot);
                    }
                }

                let inter_dst = unwrap_or!(intersections.get(lane.dst), continue);
                let inter_src = unwrap_or!(intersections.get(lane.src), continue);

                next.extend(
                    inter_dst
                        .turns_from(lane.id)
                        .map(|(turn, _)| PtrCmp(&map.lanes()[turn.dst])),
                );

                next.extend(
                    inter_src
                        .turns_to(lane.id)
                        .map(|(turn, _)| PtrCmp(&map.lanes()[turn.src])),
                )
            }
            std::mem::swap(&mut potential, &mut next);
        }
        None
    }
}
