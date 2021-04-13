use common::PtrCmp;
use geom::Vec2;
use map_model::{LaneKind, Map, ParkingSpot, ParkingSpotID, ParkingSpots};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SpotReservation(ParkingSpotID);

register_resource!(ParkingManagement, "pmanagement");
#[derive(Default, Serialize, Deserialize)]
pub struct ParkingManagement {
    reserved_spots: BTreeSet<ParkingSpotID>,
}

impl ParkingManagement {
    pub fn free(&mut self, spot: SpotReservation) {
        if !self.reserved_spots.remove(&spot.0) {
            log::warn!("{:?} wasn't reserved", spot.0);
        }
        std::mem::forget(spot);
    }

    pub fn is_free(&self, spot: SpotReservation) -> bool {
        self.is_spot_free(spot.0)
    }

    pub fn is_spot_free(&self, spot: ParkingSpotID) -> bool {
        self.reserved_spots.contains(&spot)
    }

    pub fn reserve_near(&mut self, near: Vec2, map: &Map) -> Option<SpotReservation> {
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
                            return Some(SpotReservation(spot));
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

impl SpotReservation {
    pub fn exists(&self, spots: &ParkingSpots) -> bool {
        spots.contains(self.0)
    }

    pub fn get<'a>(&self, spots: &'a ParkingSpots) -> Option<&'a ParkingSpot> {
        spots.get(self.0)
    }

    pub fn park_pos(&self, map: &Map) -> Option<Vec2> {
        map.parking_to_drive_pos(self.0)
    }
}
