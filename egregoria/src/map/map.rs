use crate::map::serializing::SerializedMap;
use crate::map::{
    Building, BuildingGen, BuildingID, BuildingKind, Intersection, IntersectionID, Lane, LaneID,
    LaneKind, LanePattern, Lot, LotID, LotKind, ParkingSpotID, ParkingSpots, ProjectFilter,
    ProjectKind, Road, RoadID, RoadSegmentKind, SpatialMap, StraightRoadGen, Terrain,
};
use geom::OBB;
use geom::{Circle, Intersect, Shape, Spline3, Vec2, Vec3};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::DenseSlotMap;
use std::collections::BTreeMap;
use std::num::Wrapping;

pub type Roads = DenseSlotMap<RoadID, Road>;
pub type Lanes = DenseSlotMap<LaneID, Lane>;
pub type Intersections = DenseSlotMap<IntersectionID, Intersection>;
pub type Buildings = DenseSlotMap<BuildingID, Building>;
pub type Lots = DenseSlotMap<LotID, Lot>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct MapProject {
    pub pos: Vec3,
    pub kind: ProjectKind,
}

pub struct Map {
    pub(crate) roads: Roads,
    pub(crate) lanes: Lanes,
    pub(crate) intersections: Intersections,
    pub(crate) buildings: Buildings,
    pub(crate) lots: Lots,
    pub(crate) spatial_map: SpatialMap,
    pub(crate) bkinds: BTreeMap<BuildingKind, Vec<BuildingID>>,
    pub terrain: Terrain,
    pub parking: ParkingSpots,
    pub dirt_id: Wrapping<u32>,
}

defer_serialize!(Map, SerializedMap);

impl Default for Map {
    fn default() -> Self {
        Self::empty()
    }
}

impl Map {
    // Public API
    pub fn empty() -> Self {
        Self {
            roads: Roads::default(),
            lanes: Lanes::default(),
            intersections: Intersections::default(),
            parking: ParkingSpots::default(),
            buildings: Buildings::default(),
            lots: Lots::default(),
            terrain: Terrain::default(),
            dirt_id: Wrapping(1),
            spatial_map: SpatialMap::default(),
            bkinds: Default::default(),
        }
    }

    pub fn update_intersection(&mut self, id: IntersectionID, f: impl Fn(&mut Intersection)) {
        info!("update_intersection {:?}", id);
        self.dirt_id += Wrapping(1);

        let inter = unwrap_ret!(self.intersections.get_mut(id));
        f(inter);
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);

        self.check_invariants()
    }

    pub fn remove_intersection(&mut self, src: IntersectionID) {
        info!("remove_intersection {:?}", src);
        self.dirt_id += Wrapping(1);

        self.remove_intersection_inner(src);

        self.check_invariants()
    }

    fn remove_intersection_inner(&mut self, src: IntersectionID) {
        let inter = unwrap_ret!(self.intersections.remove(src));

        for road in inter.roads {
            let r = unwrap_cont!(self.remove_road_inner(road));
            let o = unwrap_cont!(r.other_end(src));
            self.invalidate(o);
        }

        self.spatial_map.remove(src);
    }

    pub fn remove_building(&mut self, b: BuildingID) -> Option<Building> {
        info!("remove_building {:?}", b);

        let b = self.buildings.remove(b)?;
        self.spatial_map.remove(b.id);

        self.dirt_id += Wrapping(1);

        if b.kind.is_cached_in_bkinds() {
            self.bkinds
                .entry(b.kind)
                .and_modify(|v| v.retain(|id| *id != b.id));
        }

        self.check_invariants();

        Some(b)
    }

    pub fn make_connection(
        &mut self,
        from: MapProject,
        to: MapProject,
        interpoint: Option<Vec2>,
        pattern: &LanePattern,
    ) -> Option<(IntersectionID, RoadID)> {
        if !from.kind.check_valid(self) || !to.kind.check_valid(self) {
            return None;
        }

        let connection_segment = match interpoint {
            Some(x) => RoadSegmentKind::from_elbow(from.pos.xy(), to.pos.xy(), x),
            None => RoadSegmentKind::Straight,
        };

        let mut mk_inter = |proj: MapProject| {
            Some(match proj.kind {
                ProjectKind::Ground => self.add_intersection(proj.pos),
                ProjectKind::Inter(id) => id,
                ProjectKind::Road(id) => self.split_road(id, proj.pos)?,
                ProjectKind::Building(_) | ProjectKind::Lot(_) => unreachable!(),
            })
        };

        let from = mk_inter(from)?;
        let to = mk_inter(to)?;

        let r = self.connect(from, to, pattern, connection_segment)?;

        self.check_invariants();

        Some((to, r))
    }

    pub fn build_special_building(
        &mut self,
        obb: &OBB,
        kind: BuildingKind,
        gen: BuildingGen,
        attachments_gen: &[StraightRoadGen],
    ) -> Option<BuildingID> {
        if self.building_overlaps(*obb) {
            log::warn!("did not build {:?}: building overlaps", kind);
            return None;
        }
        log::info!(
            "build special {:?} with shape {:?} and gen {:?}",
            kind,
            obb,
            gen
        );
        self.dirt_id += Wrapping(1);
        let to_clean: Vec<_> = self.spatial_map.query(obb, ProjectFilter::LOT).collect();
        for id in to_clean {
            if let ProjectKind::Lot(id) = id {
                self.spatial_map.remove(id);

                unwrap_contlog!(
                    &self.lots.remove(id),
                    "Lot was present in spatial map but not in Lots struct"
                );
            }
        }

        let tree_remove_mask = obb.expand(10.0);
        self.terrain
            .remove_near_filter(obb.bbox(), |p| tree_remove_mask.contains(p));

        let mut attachments = vec![];
        for attachment in attachments_gen {
            let fromi = self.add_intersection(attachment.from);
            let toi = self.add_intersection(attachment.to);
            attachments.push(
                self.connect(fromi, toi, &attachment.pattern, RoadSegmentKind::Straight)
                    .unwrap(),
            );
        }

        let v = Building::make(
            &mut self.buildings,
            &mut self.spatial_map,
            &self.terrain,
            *obb,
            kind,
            gen,
            attachments,
        );

        if kind.is_cached_in_bkinds() {
            if let Some(id) = v {
                self.bkinds.entry(kind).or_default().push(id);
            }
        }

        self.check_invariants();
        v
    }

    pub fn build_house(&mut self, id: LotID) -> Option<BuildingID> {
        info!("build house on {:?}", id);
        self.dirt_id += Wrapping(1);

        let lot = self.lots.remove(id)?;

        self.spatial_map.remove(lot.id);

        let v = Building::make(
            &mut self.buildings,
            &mut self.spatial_map,
            &self.terrain,
            lot.shape,
            BuildingKind::House,
            BuildingGen::House,
            vec![],
        );
        self.check_invariants();
        v
    }

    pub fn remove_road(&mut self, road_id: RoadID) -> Option<Road> {
        info!("remove_road {:?}", road_id);

        self.dirt_id += Wrapping(1);

        let v = self.remove_road_inner(road_id);
        self.check_invariants();
        v
    }

    fn remove_road_inner(&mut self, road_id: RoadID) -> Option<Road> {
        let road = self.remove_raw_road(road_id)?;

        for (id, _) in road.lanes_iter() {
            self.parking.remove_spots(id);
        }

        if road.lanes_iter().any(|(_, v)| v.is_rail()) {
            self.buildings
                .retain(|_, t| !t.attachments.contains(&road_id));
        }

        let smap = &mut self.spatial_map;
        self.lots.retain(|_, lot| {
            let to_remove = lot.parent == road_id;
            if to_remove {
                smap.remove(lot.id);
            }
            !to_remove
        });

        self.invalidate(road.src);
        self.invalidate(road.dst);
        Some(road)
    }

    pub fn set_lot_kind(&mut self, lot: LotID, kind: LotKind) {
        match self.lots.get_mut(lot) {
            Some(lot) => {
                lot.kind = kind;
                self.dirt_id += Wrapping(1);
            }
            None => log::warn!("trying to set kind of non-existing lot {:?}", lot),
        }
    }

    pub fn clear(&mut self) {
        info!("clear");
        let before = std::mem::replace(self, Self::empty());
        self.terrain = before.terrain;
        self.dirt_id = before.dirt_id + Wrapping(1);

        self.check_invariants();
    }

    // Private mutating

    pub(crate) fn add_intersection(&mut self, pos: Vec3) -> IntersectionID {
        Intersection::make(&mut self.intersections, &mut self.spatial_map, pos)
    }

    fn invalidate(&mut self, id: IntersectionID) {
        info!("invalidate {:?}", id);

        self.dirt_id += Wrapping(1);
        let inter = unwrap_ret!(self.intersections.get_mut(id));

        if inter.roads.is_empty() {
            self.remove_intersection_inner(id);
            return;
        }

        inter.update_interface_radius(&mut self.roads);

        for x in inter.roads.clone() {
            let road = unwrap_contlog!(
                self.roads.get(x),
                "intersection has unexisting road in list"
            );

            let oend_id = unwrap_cont!(road.other_end(id));

            let other_end = unwrap_contlog!(
                self.intersections.get_mut(oend_id),
                "road is connected to unexisting intersection"
            );
            other_end.update_interface_radius(&mut self.roads);

            #[allow(clippy::indexing_slicing)] // borrowed before
            self.roads[x].update_lanes(&mut self.lanes, &mut self.parking);
        }

        #[allow(clippy::indexing_slicing)] // borrowed before
        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);

        self.spatial_map
            .update(inter.id, inter.bcircle(&self.roads));
    }

    /// Only removes road from Roads and spatial map but keeps lots,
    /// and potentially empty intersections.
    fn remove_raw_road(&mut self, road_id: RoadID) -> Option<Road> {
        let road = self.roads.remove(road_id)?;

        self.spatial_map.remove(road_id);

        for (id, _) in road.lanes_iter() {
            self.lanes.remove(id);
        }

        if let Some(i) = self.intersections.get_mut(road.src) {
            i.remove_road(road_id);
        }
        if let Some(i) = self.intersections.get_mut(road.dst) {
            i.remove_road(road_id);
        }

        Some(road)
    }

    #[allow(clippy::collapsible_else_if)]
    pub(crate) fn split_road(&mut self, r_id: RoadID, pos: Vec3) -> Option<IntersectionID> {
        info!("split_road {:?} {:?}", r_id, pos);

        let pat = self.roads.get(r_id)?.pattern(&self.lanes);

        let r = unwrap_or!(self.remove_raw_road(r_id), {
            log::error!("Trying to split unexisting road");
            return None;
        });

        for (id, _) in r.lanes_iter() {
            self.parking.remove_to_reuse(id);
        }

        let id = self.add_intersection(pos);

        let src_id = r.src;

        let (r1, r2) = match r.segment {
            RoadSegmentKind::Straight => (
                self.connect(src_id, id, &pat, RoadSegmentKind::Straight)?,
                self.connect(id, r.dst, &pat, RoadSegmentKind::Straight)?,
            ),
            RoadSegmentKind::Curved((from_derivative, to_derivative)) => {
                let s = Spline3 {
                    from: r.points.first(),
                    to: r.points.last(),
                    from_derivative: from_derivative.z0(),
                    to_derivative: to_derivative.z0(),
                };
                let t_approx = s.project_t(pos, 1.0);

                let (s_from, s_to) = s.split_at(t_approx);

                (
                    self.connect(
                        src_id,
                        id,
                        &pat,
                        RoadSegmentKind::Curved((
                            s_from.from_derivative.xy(),
                            s_from.to_derivative.xy(),
                        )),
                    )?,
                    self.connect(
                        id,
                        r.dst,
                        &pat,
                        RoadSegmentKind::Curved((
                            s_to.from_derivative.xy(),
                            s_to.to_derivative.xy(),
                        )),
                    )?,
                )
            }
        };

        log::info!(
            "{} parking spots reused when splitting",
            self.parking.clean_reuse()
        );

        let r1 = self.roads.get(r1)?;
        let r2 = self.roads.get(r2)?;

        let spatial = &mut self.spatial_map;
        self.lots.retain(|_, lot| {
            if lot.parent != r_id {
                return true;
            }
            let p = lot.shape.corners[0].z(lot.height);
            let d1 = r1.points.project(p).distance(p);
            let d2 = r2.points.project(p).distance(p);
            if d1 < d2 {
                if d1 < r1.width * 0.5 + 1.5 {
                    lot.parent = r1.id;
                } else {
                    spatial.remove(lot.id);
                    return false;
                }
            } else {
                if d2 < r2.width * 0.5 + 1.5 {
                    lot.parent = r2.id;
                } else {
                    spatial.remove(lot.id);
                    return false;
                }
            }
            true
        });

        Some(id)
    }

    /// Returns None if one of the intersections don't exist
    pub(crate) fn connect(
        &mut self,
        src_id: IntersectionID,
        dst_id: IntersectionID,
        pattern: &LanePattern,
        segment: RoadSegmentKind,
    ) -> Option<RoadID> {
        info!(
            "connect {:?} {:?} {:?} {:?}",
            src_id, dst_id, pattern, segment
        );
        self.dirt_id += Wrapping(1);

        let src = self.intersections.get(src_id)?;
        let dst = self.intersections.get(dst_id)?;

        let id = Road::make(
            src,
            dst,
            segment,
            pattern,
            &mut self.roads,
            &mut self.lanes,
            &mut self.parking,
            &mut self.spatial_map,
        );
        #[allow(clippy::indexing_slicing)]
        let r = &self.roads[id];

        self.intersections.get_mut(src_id)?.add_road(&self.roads, r);
        self.intersections.get_mut(dst_id)?.add_road(&self.roads, r);

        self.invalidate(src_id);
        self.invalidate(dst_id);

        Lot::remove_intersecting_lots(self, id);
        Lot::generate_along_road(self, id);

        #[allow(clippy::indexing_slicing)]
        let r = &self.roads[id];
        let d = r.width + 50.0;
        let b = r.boldline();
        self.terrain.remove_near_filter(b.bbox().expand(d), |tpos| {
            let rd = common::rand::rand3(tpos.x, tpos.y, 391.0) * 20.0;
            b.intersects(&Circle::new(tpos, d - rd))
        });

        Some(id)
    }

    // Public helpers
    pub fn project(&self, pos: Vec3, tolerance: f32, filter: ProjectFilter) -> MapProject {
        let mk_proj = move |kind| MapProject { pos, kind };

        let mut qroad = None;
        for pkind in self.spatial_map.query_around(pos.xy(), tolerance, filter) {
            match pkind {
                ProjectKind::Inter(id) => {
                    let inter = unwrap_contlog!(self.intersections.get(id),
                        "Inter does not exist anymore, you seem to have forgotten to remove it from the spatial map.");

                    return MapProject {
                        pos: inter.pos,
                        kind: pkind,
                    };
                }
                ProjectKind::Lot(id) => {
                    return mk_proj(ProjectKind::Lot(id));
                }
                ProjectKind::Road(id) => {
                    if qroad.is_some() {
                        continue;
                    }
                    let road = unwrap_contlog!(self.roads.get(id),
                        "Road does not exist anymore, you seem to have forgotten to remove it from the spatial map.");

                    let projected = road.points.project(pos);
                    qroad = Some((id, projected));
                }
                ProjectKind::Building(id) => {
                    return mk_proj(ProjectKind::Building(id));
                }
                ProjectKind::Ground => {}
            }
        }

        if let Some((id, pos)) = qroad {
            return MapProject {
                pos,
                kind: ProjectKind::Road(id),
            };
        }

        mk_proj(ProjectKind::Ground)
    }

    pub fn is_empty(&self) -> bool {
        self.roads.is_empty() && self.lanes.is_empty() && self.intersections.is_empty()
    }

    pub fn roads(&self) -> &Roads {
        &self.roads
    }
    pub fn lanes(&self) -> &Lanes {
        &self.lanes
    }
    pub fn intersections(&self) -> &Intersections {
        &self.intersections
    }
    pub fn buildings(&self) -> &Buildings {
        &self.buildings
    }
    pub fn lots(&self) -> &Lots {
        &self.lots
    }
    pub fn spatial_map(&self) -> &SpatialMap {
        &self.spatial_map
    }

    pub fn building_overlaps(&self, obb: OBB) -> bool {
        self.spatial_map
            .query(obb, ProjectFilter::BUILDING)
            .next()
            .is_some()
    }

    pub fn find_road(&self, src: IntersectionID, dst: IntersectionID) -> Option<RoadID> {
        for &r in &self.intersections.get(src)?.roads {
            let road = unwrap_cont!(self.roads.get(r));
            if road.src == src && road.dst == dst {
                return Some(road.id);
            }
        }
        None
    }

    pub fn nearest_lane(&self, p: Vec3, kind: LaneKind, cutoff: Option<f32>) -> Option<LaneID> {
        let tryfind = |radius| {
            self.spatial_map()
                .query_around(p.xy(), radius, ProjectFilter::ROAD)
                .filter_map(|x| {
                    if let ProjectKind::Road(id) = x {
                        Some(id)
                    } else {
                        unsafe { std::hint::unreachable_unchecked() }
                    }
                })
                .filter_map(|id| self.roads().get(id))
                .flat_map(|road| road.lanes_iter())
                .filter(|&(_, x)| x == kind)
                .map(|(id, _)| &self.lanes[id])
                .min_by_key(|lane| OrderedFloat(lane.points.project_dist2(p)))
        };

        if let Some(cutoff) = cutoff {
            return tryfind(cutoff).map(|v| v.id);
        }

        if let Some(lane) = tryfind(20.0) {
            return Some(lane.id);
        }

        if let Some(lane) = tryfind(100.0) {
            return Some(lane.id);
        }

        self.lanes
            .iter()
            .filter(|(_, x)| x.kind == kind)
            .min_by_key(|(_, lane)| OrderedFloat(lane.points.project_dist2(p)))
            .map(|(id, _)| id)
    }

    pub fn parking_to_drive(&self, spot: ParkingSpotID) -> Option<LaneID> {
        let spot = self.parking.get(spot)?;
        let park_lane = self.lanes.get(spot.parent)?;
        let road = self.roads.get(park_lane.parent)?;
        road.outgoing_lanes_from(park_lane.src)
            .iter()
            .rfind(|&&(_, kind)| kind == LaneKind::Driving)
            .map(|&(id, _)| id)
    }

    pub fn parking_to_drive_pos(&self, spot: ParkingSpotID) -> Option<Vec3> {
        let spot = self.parking.get(spot)?;
        let park_lane = self.lanes.get(spot.parent)?;
        let road = self.roads.get(park_lane.parent)?;
        let lane = road
            .outgoing_lanes_from(park_lane.src)
            .iter()
            .rfind(|&&(_, kind)| kind == LaneKind::Driving)
            .map(|&(id, _)| id)?;

        let (pos, _, dir) = self
            .lanes()
            .get(lane)?
            .points
            .project_segment_dir(spot.trans.position);
        Some(pos - dir * 4.0)
    }

    #[cfg(not(debug_assertions))]
    pub fn check_invariants(&self) {}

    #[cfg(debug_assertions)]
    pub fn check_invariants(&self) {
        if std::env::var("MAP_INVARIANT_CHECK").is_err() {
            return;
        }
        for inter in self.intersections.values() {
            log::debug!("{:?}", inter.id);
            assert!(!inter.roads.is_empty());

            let mut last_angle = -f32::INFINITY;
            for &road in &inter.roads {
                let road = self.roads.get(road).expect("road does not exist");
                let ang = geom::pseudo_angle(road.dir_from(inter.id));
                assert!(ang > last_angle);
                last_angle = ang;
            }

            for turn in inter.turns() {
                log::debug!("{:?}", turn.id);
                assert_eq!(turn.id.parent, inter.id);
                assert!(self.lanes.contains_key(turn.id.src));
                assert!(self.lanes.contains_key(turn.id.dst));
                assert!(turn.points.n_points() >= 2);
            }

            assert!(inter.pos.is_finite());
            assert!(self.spatial_map.contains(inter.id));
        }

        for lane in self.lanes.values() {
            log::debug!("{:?}", lane.id);
            assert!(!lane.points.is_empty());
            assert!(self.intersections.contains_key(lane.src), "{:?}", lane.src);
            assert!(self.intersections.contains_key(lane.dst), "{:?}", lane.dst);
            assert!(self.roads.contains_key(lane.parent), "{:?}", lane.parent);

            if matches!(lane.kind, LaneKind::Parking) {
                assert!(
                    self.parking.spots(lane.id).is_some(),
                    "no spots for {:?}",
                    lane.id
                )
            }
        }

        for bs in self.bkinds.values() {
            for &b in bs {
                assert!(self.buildings.contains_key(b));
            }
        }

        for road in self.roads.values() {
            log::debug!("{:?}", road.id);
            let src = self.intersections.get(road.src).unwrap();
            assert!(src.roads.contains(&road.id));
            let dst = self.intersections.get(road.dst).unwrap();
            assert!(dst.roads.contains(&road.id));
            assert!(!road.points.is_empty());
            assert!(road.lanes_iter().next().is_some());
            assert!(road.points.first().is_close(src.pos, 0.001),);
            assert!(road.points.last().is_close(dst.pos, 0.001));
            assert!(road.interfaced_points().n_points() >= 2);
            assert!(road.length() > 0.0);
            assert!(self.spatial_map.contains(road.id));

            for (id, _) in road.lanes_iter() {
                let v = self.lanes.get(id).expect("lane child does not exist");
                assert_eq!(v.parent, road.id);
            }

            // Road with parking lane has driving lane (incoming)
            let has_parking = road
                .incoming_lanes_to(road.src)
                .iter()
                .any(|(_, kind)| matches!(kind, LaneKind::Parking));

            if has_parking {
                let has_driving = road
                    .incoming_lanes_to(road.src)
                    .iter()
                    .any(|(_, kind)| matches!(kind, LaneKind::Driving));
                assert!(has_driving);
            }

            // Road with parking lane has driving lane (outgoing)
            let has_parking = road
                .outgoing_lanes_from(road.src)
                .iter()
                .any(|(_, kind)| matches!(kind, LaneKind::Parking));

            if has_parking {
                let has_driving = road
                    .outgoing_lanes_from(road.src)
                    .iter()
                    .any(|(_, kind)| matches!(kind, LaneKind::Driving));
                assert!(has_driving);
            }
        }

        for lot in self.lots.values() {
            log::debug!("{:?}", lot.id);
            assert!(lot.shape.axis().iter().all(|x| x.mag() > 0.0));
            assert!(self.roads.contains_key(lot.parent), "{:?}", lot.parent);
            assert!(self.spatial_map.contains(lot.id));
        }

        for obj in self.spatial_map.objects() {
            assert!(self.spatial_map.contains(*obj));
            log::debug!("{:?}", obj);
            assert!(obj.check_valid(self));
        }

        assert!(self.parking.reuse_spot.is_empty());
    }
}

impl MapProject {
    pub fn ground(pos: Vec3) -> Self {
        Self {
            pos,
            kind: ProjectKind::Ground,
        }
    }
}
