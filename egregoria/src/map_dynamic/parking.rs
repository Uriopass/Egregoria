use common::PtrCmp;
use geom::Vec2;
use map_model::{LaneKind, Map, ParkingSpotID};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

register_resource!(ParkingManagement, "pmanagement");
#[derive(Default, Serialize, Deserialize)]
pub struct ParkingManagement {
    reserved_spots: BTreeSet<ParkingSpotID>,
}

impl ParkingManagement {
    pub fn free(&mut self, spot: ParkingSpotID) {
        if !self.reserved_spots.remove(&spot) {
            log::warn!("{:?} wasn't reserved", spot);
        }
    }

    pub fn is_free(&self, spot: ParkingSpotID) -> bool {
        self.reserved_spots.contains(&spot)
    }

    pub fn reserve_near(&mut self, near: Vec2, map: &Map) -> Option<ParkingSpotID> {
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

                if let Some(p_iter) = map.parking.closest_spots(plane, near) {
                    for spot in p_iter {
                        if self.reserved_spots.insert(spot) {
                            return Some(spot);
                        }
                    }
                }

                let inter_dst = unwrap_or!(intersections.get(lane.dst), continue);
                let inter_src = unwrap_or!(intersections.get(lane.src), continue);

                next.extend(
                    inter_dst
                        .turns_from(lane.id)
                        .flat_map(|(turn, _)| Some(PtrCmp(map.lanes().get(turn.dst)?))),
                );

                next.extend(
                    inter_src
                        .turns_to(lane.id)
                        .flat_map(|(turn, _)| Some(PtrCmp(map.lanes().get(turn.src)?))),
                )
            }
            std::mem::swap(&mut potential, &mut next);
        }
        None
    }
}
