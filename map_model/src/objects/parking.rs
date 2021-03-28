use crate::{Lane, LaneID, LaneKind, CROSSWALK_WIDTH};
use flat_spatial::ShapeGrid;
use geom::{Transform, Vec2};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

new_key_type! {
    pub struct ParkingSpotID;
}

pub const PARKING_SPOT_LENGTH: f32 = 6.0;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ParkingSpot {
    pub parent: LaneID,
    pub trans: Transform,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParkingSpots {
    spots: SlotMap<ParkingSpotID, ParkingSpot>,
    lane_spots: SecondaryMap<LaneID, Vec<ParkingSpotID>>,
    pub(crate) reuse_spot: ShapeGrid<ParkingSpotID, Vec2>,
}

impl Default for ParkingSpots {
    fn default() -> Self {
        Self {
            spots: Default::default(),
            lane_spots: Default::default(),
            reuse_spot: ShapeGrid::new(10),
        }
    }
}

impl ParkingSpots {
    pub fn get(&self, spot: ParkingSpotID) -> Option<&ParkingSpot> {
        self.spots.get(spot)
    }

    pub fn contains(&self, spot: ParkingSpotID) -> bool {
        self.spots.contains_key(spot)
    }

    pub fn remove_spots(&mut self, lane: LaneID) {
        if let Some(spots) = self.lane_spots.remove(lane) {
            for spot in spots {
                self.spots.remove(spot);
            }
        }
    }

    pub fn clean_reuse(&mut self) -> u32 {
        let mut has_reused = 0;
        for (_, spot) in self.reuse_spot.clear() {
            self.spots.remove(spot);
            has_reused += 1;
        }
        has_reused
    }

    pub fn remove_to_reuse(&mut self, lane: LaneID) {
        if let Some(spots) = self.lane_spots.remove(lane) {
            for spot_id in spots {
                let spot = unwrap_cont!(self.spots.get(spot_id));
                self.reuse_spot.insert(spot.trans.position(), spot_id);
            }
        }
    }

    pub fn generate_spots(&mut self, lane: &Lane) {
        debug_assert!(matches!(lane.kind, LaneKind::Parking));

        if self.lane_spots.contains_key(lane.id) {
            self.remove_to_reuse(lane.id);
        }

        let gap = CROSSWALK_WIDTH + 4.0;
        let l = lane.length() - gap * 2.0;
        let n_spots = (l / PARKING_SPOT_LENGTH) as i32;
        if n_spots <= 0 {
            return;
        }
        let step = l / n_spots as f32;

        let parent = lane.id;
        let spots = &mut self.spots;
        let reuse = &mut self.reuse_spot;
        let spots = lane
            .points
            .points_dirs_along((0..n_spots).map(|x| (x as f32 + 0.5) * step + gap))
            .map(move |(pos, dir)| {
                let mut iter = reuse.query_around(pos, 3.0);
                if let Some(h) = iter.next().map(|x| x.0) {
                    drop(iter);

                    let spot_id = reuse.remove(h).unwrap();
                    if let Some(p) = spots.get_mut(spot_id) {
                        *p = ParkingSpot {
                            parent,
                            trans: Transform::new_cos_sin(pos, dir),
                        };
                        return spot_id;
                    } else {
                        log::error!("found a spot in reuse that doesn't exist anymore");
                    }
                }

                spots.insert(ParkingSpot {
                    parent,
                    trans: Transform::new_cos_sin(pos, dir),
                })
            })
            .collect();

        self.lane_spots.insert(lane.id, spots);
    }

    pub fn clear(&mut self) {
        self.spots.clear();
        self.lane_spots.clear();
    }

    pub fn spots(&self, lane: LaneID) -> impl Iterator<Item = &ParkingSpot> + '_ {
        self.lane_spots
            .get(lane)
            .map(move |x| x.iter().flat_map(move |spot| self.spots.get(*spot)))
            .into_iter()
            .flatten()
    }

    pub fn all_spots(&self) -> impl Iterator<Item = (ParkingSpotID, &ParkingSpot)> + '_ {
        self.spots.iter()
    }

    // Fixme: Instead of allocating a vec and sorting it, somehow sort the parking spots beforehand and iterate in spiral around the projected `near`
    pub fn closest_spots(&self, lane: LaneID, near: Vec2) -> impl Iterator<Item = ParkingSpotID> {
        let spots = &self.spots;
        let mut lspots = self.lane_spots.get(lane).cloned();
        if let Some(ref mut lspots) = lspots {
            lspots.sort_by_key(|&id| OrderedFloat(spots[id].trans.position().distance2(near)))
        }
        lspots.into_iter().flatten()
    }
}
