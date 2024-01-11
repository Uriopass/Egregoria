use crate::map::{Lane, LaneID, LaneKind, CROSSWALK_WIDTH};
use flat_spatial::Grid;
use geom::{Transform, Vec2, Vec3};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmapd::{new_key_type, SecondaryMap, SlotMap};

new_key_type! {
    pub struct ParkingSpotID;
}

pub const PARKING_SPOT_LENGTH: f32 = 6.0;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ParkingSpot {
    pub parent: LaneID,
    pub trans: Transform,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParkingSpots {
    pub(crate) spots: SlotMap<ParkingSpotID, ParkingSpot>,
    pub(crate) lane_spots: SecondaryMap<LaneID, Vec<ParkingSpotID>>,
    pub(crate) reuse_spot: Grid<ParkingSpotID, Vec2>,
}

impl Default for ParkingSpots {
    fn default() -> Self {
        Self {
            spots: Default::default(),
            lane_spots: Default::default(),
            reuse_spot: Grid::new(10),
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

    pub fn random_spot(&self, rng: u64) -> Option<ParkingSpotID> {
        if self.spots.is_empty() {
            return None;
        }
        let mut iter = self.spots.keys();
        iter.nth(rng as usize % self.spots.len())
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
                self.reuse_spot.insert(spot.trans.pos.xy(), spot_id);
            }
        }
    }

    pub fn generate_spots(&mut self, lane: &Lane) {
        debug_assert!(matches!(lane.kind, LaneKind::Parking));
        if self.lane_spots.contains_key(lane.id) {
            self.remove_to_reuse(lane.id);
        }

        let gap = CROSSWALK_WIDTH + 1.0;
        let l = lane.points.length() - gap * 2.0;
        let n_spots = (l / PARKING_SPOT_LENGTH) as i32;
        if n_spots <= 0 {
            self.lane_spots.insert(lane.id, vec![]);
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
                let mut iter = reuse.query_around(pos.xy(), 3.0);
                if let Some((h, _)) = iter.next() {
                    if let Some((_, spot_id)) = reuse.get(h) {
                        let spot_id = *spot_id;
                        drop(iter);

                        reuse.remove_maintain(h);
                        if let Some(p) = spots.get_mut(spot_id) {
                            *p = ParkingSpot {
                                parent,
                                trans: Transform::new_dir(pos, dir),
                            };
                            return spot_id;
                        } else {
                            log::error!("found a spot in reuse that doesn't exist anymore");
                        }
                    }
                }

                spots.insert(ParkingSpot {
                    parent,
                    trans: Transform::new_dir(pos, dir),
                })
            })
            .collect();

        self.lane_spots.insert(lane.id, spots);
    }

    pub fn clear(&mut self) {
        self.spots.clear();
        self.lane_spots.clear();
        for _ in self.reuse_spot.clear() {}
    }

    pub fn spots(&self, lane: LaneID) -> Option<impl Iterator<Item = &ParkingSpot> + '_> {
        self.lane_spots
            .get(lane)
            .map(move |x| x.iter().flat_map(move |spot| self.spots.get(*spot)))
    }

    pub fn all_spots(&self) -> impl Iterator<Item = (ParkingSpotID, &ParkingSpot)> + '_ {
        self.spots.iter()
    }

    /// Iterate in spiral around the projected `near`
    pub fn closest_spots(
        &self,
        lane: LaneID,
        near: Vec3,
    ) -> Option<impl Iterator<Item = ParkingSpotID> + '_> {
        let spots = &self.spots;
        let lspots = self.lane_spots.get(lane)?;
        let (closest, _) = lspots.iter().copied().enumerate().min_by_key(|&(_, x)| {
            let p = unwrap_ret!(spots.get(x), OrderedFloat(f32::INFINITY));
            OrderedFloat(p.trans.pos.distance2(near))
        })?;

        let closest = closest as i32;

        Some(
            (0..lspots.len() as i32 * 2)
                .map(|i| if i & 1 == 0 { i >> 1 } else { -(i >> 1) })
                .filter_map(move |offset| {
                    let i = unwrap_orr!(usize::try_from(closest + offset), return None);
                    lspots.get(i)
                })
                .copied(),
        )
    }
}
