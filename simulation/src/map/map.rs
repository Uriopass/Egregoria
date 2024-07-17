use crate::map::electricity_cache::ElectricityCache;
use crate::map::height_override::find_overrides;
use crate::map::serializing::SerializedMap;
use crate::map::{
    Building, BuildingID, BuildingKind, Environment, Intersection, IntersectionID, Lane, LaneID,
    LaneKind, LanePattern, Lot, LotID, LotKind, MapSubscriber, MapSubscribers, ParkingSpotID,
    ParkingSpots, ProjectFilter, ProjectKind, Road, RoadID, RoadSegmentKind, SpatialMap,
    SubscriberChunkID, TerraformKind, UpdateType, Zone, ROAD_Z_OFFSET,
};
use geom::OBB;
use geom::{Vec2, Vec3};
use ordered_float::OrderedFloat;
use prototypes::{BuildingGen, Tick};
use serde::{Deserialize, Serialize};
use slotmapd::HopSlotMap;

pub type Roads = HopSlotMap<RoadID, Road>;
pub type Lanes = HopSlotMap<LaneID, Lane>;
pub type Intersections = HopSlotMap<IntersectionID, Intersection>;
pub type Buildings = HopSlotMap<BuildingID, Building>;
pub type Lots = HopSlotMap<LotID, Lot>;

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
    pub(crate) external_train_stations: Vec<BuildingID>,

    pub electricity: ElectricityCache,
    pub environment: Environment,
    pub parking: ParkingSpots,
    pub subscribers: MapSubscribers,
    pub(crate) override_subscriber: MapSubscriber,
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
        let subscribers = MapSubscribers::default();
        Self {
            roads: Roads::default(),
            lanes: Lanes::default(),
            intersections: Intersections::default(),
            parking: ParkingSpots::default(),
            buildings: Buildings::default(),
            lots: Lots::default(),
            environment: Environment::default(),
            spatial_map: SpatialMap::default(),
            external_train_stations: Default::default(),
            electricity: Default::default(),
            override_subscriber: subscribers.subscribe(UpdateType::Road | UpdateType::Building),
            subscribers,
        }
    }

    /// Perform cleanups potentially required every frame
    pub fn update(&mut self) {
        profiling::scope!("map::update");
        for chunk in self.override_subscriber.take_updated_chunks() {
            find_overrides(self, chunk);
        }
    }

    pub fn update_intersection(&mut self, id: IntersectionID, f: impl Fn(&mut Intersection)) {
        info!("update_intersection {:?}", id);

        let Some(inter) = self.intersections.get_mut(id) else {
            return;
        };
        f(inter);
        self.invalidate(id);

        self.check_invariants()
    }

    pub fn remove_intersection(&mut self, src: IntersectionID) {
        info!("remove_intersection {:?}", src);
        self.remove_intersection_inner(src);

        self.check_invariants()
    }

    fn remove_intersection_inner(&mut self, src: IntersectionID) {
        let inter = unwrap_ret!(self.intersections.remove(src));
        self.subscribers.dispatch(UpdateType::Road, &inter);

        for road in inter.roads {
            let r = unwrap_cont!(self.remove_road_inner(road));

            let o = unwrap_cont!(r.other_end(src));
            self.invalidate(o);
        }

        self.electricity.remove_object(src);
        self.spatial_map.remove(src);
    }

    pub fn dispatch_all(&self) {
        self.subscribers.dispatch_clear();
        let bounds = self.environment.bounds();
        let ll = SubscriberChunkID::new(bounds.ll);
        let ur = SubscriberChunkID::new(bounds.ur);
        for x in ll.0..ur.0 {
            for y in ll.0..ur.0 {
                self.subscribers
                    .dispatch_all(std::iter::once(SubscriberChunkID::new_i16(x, y)))
            }
        }
    }

    pub fn remove_building(&mut self, b: BuildingID) -> Option<Building> {
        info!("remove_building {:?}", b);

        let b = self.buildings.remove(b)?;
        self.subscribers.dispatch(UpdateType::Building, &b);

        if b.kind == BuildingKind::ExternalTrading {
            self.external_train_stations.retain(|id| *id != b.id);
        }

        if let Some(r) = b.connected_road {
            self.roads[r].connected_buildings.retain(|x| *x != b.id);
        }

        self.electricity.remove_object(b.id);
        self.spatial_map.remove(b.id);

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
        if !from.kind.check_valid(self)
            || !to.kind.check_valid(self)
            || from.pos.distance(to.pos) < 1.0
        {
            return None;
        }
        info!("make_connection {:?} {:?} {:?}", from, to, interpoint);

        let connection_segment = match interpoint {
            Some(x) => RoadSegmentKind::from_elbow(from.pos.xy(), to.pos.xy(), x),
            None => RoadSegmentKind::Straight,
        };

        let mut mk_inter = |proj: MapProject| {
            Some(match proj.kind {
                ProjectKind::Ground => self.add_intersection(proj.pos),
                ProjectKind::Intersection(id) => id,
                ProjectKind::Road(id) => self.split_road(id, proj.pos)?,
                ProjectKind::Building(_) | ProjectKind::Lot(_) => unreachable!(),
            })
        };

        let from_id = mk_inter(from)?;
        let Some(to_id) = mk_inter(to) else {
            self.invalidate(from_id);
            self.check_invariants();
            return None;
        };

        let Some(r) = self.connect(from_id, to_id, pattern, connection_segment) else {
            self.invalidate(from_id);
            self.invalidate(to_id);
            self.check_invariants();
            return None;
        };

        self.invalidate(from_id);
        self.invalidate(to_id);

        let mut maybe_merge = |inter_id| {
            let inter = &self.intersections[inter_id];
            if let [r1_id, r2_id] = *inter.roads {
                let r1 = &self.roads[r1_id];
                let r2 = &self.roads[r2_id];
                // only merge if angle is shallow
                if r1.dir_from(inter_id).dot(r2.dir_from(inter_id)) < -0.9 {
                    self.merge_road(r1_id, r2_id);
                }
            }
        };
        maybe_merge(from_id);
        maybe_merge(to_id);

        info!("connected {:?} {:?}: {:?}", from_id, to_id, r);

        self.check_invariants();

        Some((to_id, r))
    }

    pub fn update_zone(&mut self, id: BuildingID, f: impl FnOnce(&mut Zone)) {
        let Some(b) = self.buildings.get_mut(id) else {
            return;
        };
        self.subscribers.dispatch(UpdateType::Building, b);

        let Some(ref mut z) = b.zone else {
            return;
        };
        f(z);

        self.environment.remove_trees_near(&z.poly, |tree_chunk| {
            self.subscribers
                .dispatch_chunk(UpdateType::Terrain, tree_chunk)
        });

        let toclean = self
            .spatial_map
            .query(&z.poly, ProjectFilter::LOT)
            .collect();

        self.spatial_map.update(b);

        self.clean_lots_inner(toclean);

        self.check_invariants()
    }

    pub fn build_special_building(
        &mut self,
        obb: &OBB,
        kind: BuildingKind,
        gen: BuildingGen,
        zone: Option<Zone>,
        connected_road: Option<RoadID>,
    ) -> Option<BuildingID> {
        if self.building_overlaps(*obb) {
            log::warn!("did not build {:?}: building overlaps", kind);
            return None;
        }
        log::info!(
            "build special {:?} with shape {:?} and gen {:?} and zone {:?}",
            kind,
            obb,
            gen,
            zone
        );

        self.clean_lots_inner(self.spatial_map.query(obb, ProjectFilter::LOT).collect());

        self.environment
            .remove_trees_near(obb.expand(10.0), |tree_chunk| {
                self.subscribers
                    .dispatch_chunk(UpdateType::Terrain, tree_chunk)
            });

        let Some(id) = Building::make(
            &mut self.buildings,
            &mut self.spatial_map,
            &mut self.electricity,
            &mut self.roads,
            &self.environment,
            *obb,
            kind,
            gen,
            zone,
            connected_road,
        ) else {
            self.check_invariants();
            return None;
        };

        self.subscribers
            .dispatch(UpdateType::Building, &self.buildings[id]);

        if kind == BuildingKind::ExternalTrading {
            self.external_train_stations.push(id);
        }

        self.check_invariants();
        Some(id)
    }

    pub fn build_house(&mut self, lot_id: LotID) -> Option<BuildingID> {
        info!("build house on {:?}", lot_id);

        let lot = self.lots.remove(lot_id)?;
        self.subscribers.dispatch(UpdateType::Road, &lot);
        self.spatial_map.remove(lot.id);

        let Some(id) = Building::make(
            &mut self.buildings,
            &mut self.spatial_map,
            &mut self.electricity,
            &mut self.roads,
            &self.environment,
            lot.shape,
            BuildingKind::House,
            BuildingGen::House,
            None,
            Some(lot.parent),
        ) else {
            self.check_invariants();
            return None;
        };

        self.subscribers
            .dispatch(UpdateType::Building, &self.buildings[id]);
        self.electricity.add_object(id);

        self.check_invariants();
        Some(id)
    }

    pub fn remove_road(&mut self, road_id: RoadID) -> Option<Road> {
        info!("remove_road {:?}", road_id);

        let v = self.remove_road_inner(road_id);
        self.check_invariants();
        v
    }

    pub fn subscribe(&self, filter: UpdateType) -> MapSubscriber {
        self.subscribers.subscribe(filter)
    }

    fn clean_lots_inner(&mut self, to_clean: Vec<ProjectKind>) {
        for id in to_clean {
            if let ProjectKind::Lot(id) = id {
                self.spatial_map.remove(id);
                self.subscribers.dispatch(UpdateType::Road, &self.lots[id]);

                unwrap_contlog!(
                    &self.lots.remove(id),
                    "Lot was present in spatial map but not in Lots struct"
                );
            }
        }
    }

    fn remove_road_inner(&mut self, road_id: RoadID) -> Option<Road> {
        let road = self.remove_raw_road(road_id)?;
        self.subscribers.dispatch(UpdateType::Road, &road);

        for (id, _) in road.lanes_iter() {
            self.parking.remove_spots(id);
        }

        let smap = &mut self.spatial_map;
        self.lots.retain(|_, lot| {
            let to_remove = lot.parent == road_id;
            if to_remove {
                self.subscribers.dispatch(UpdateType::Road, lot);
                smap.remove(lot.id);
            }
            !to_remove
        });

        self.invalidate(road.src);
        self.invalidate(road.dst);

        for &building_id in &road.connected_buildings {
            self.buildings[building_id].connected_road = None;
        }

        Some(road)
    }

    pub fn set_lot_kind(&mut self, lot: LotID, kind: LotKind) {
        match self.lots.get_mut(lot) {
            Some(lot) => {
                lot.kind = kind;
                self.subscribers.dispatch(UpdateType::Road, lot);
            }
            None => log::warn!("trying to set kind of non-existing lot {:?}", lot),
        }
    }

    pub fn terraform(
        &mut self,
        tick: Tick,
        kind: TerraformKind,
        center: Vec2,
        radius: f32,
        amount: f32,
        level: f32,
        slope: Option<(Vec3, Vec3)>,
    ) {
        let modified = self
            .environment
            .terraform(tick, kind, center, radius, amount, level, slope);

        for id in modified {
            self.subscribers.dispatch_chunk(UpdateType::Terrain, id);
        }
    }

    // Private mutating

    pub(crate) fn add_intersection(&mut self, pos: Vec3) -> IntersectionID {
        let id = Intersection::make(&mut self.intersections, &mut self.spatial_map, pos);
        self.subscribers
            .dispatch(UpdateType::Building, &self.intersections[id]);
        self.electricity.add_object(id);
        id
    }

    fn invalidate(&mut self, id: IntersectionID) {
        info!("invalidate {:?}", id);

        let inter = unwrap_ret!(self.intersections.get_mut(id));
        self.subscribers.dispatch(UpdateType::Road, inter);

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
            self.subscribers.dispatch(UpdateType::Road, road);

            let oend_id = unwrap_cont!(road.other_end(id));

            let other_end = unwrap_contlog!(
                self.intersections.get_mut(oend_id),
                "road is connected to unexisting intersection"
            );
            other_end.update_interface_radius(&mut self.roads);

            #[allow(clippy::indexing_slicing)] // borrowed before
            self.roads[x].update_lanes(&mut self.lanes, &mut self.parking, &self.environment);
        }

        #[allow(clippy::indexing_slicing)] // borrowed before
        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);

        self.spatial_map.update(inter);
    }

    /// Only removes road from Roads and spatial map but keeps lots, buildings connection
    /// and potentially empty intersections.
    fn remove_raw_road(&mut self, road_id: RoadID) -> Option<Road> {
        let road = self.roads.remove(road_id)?;

        self.spatial_map.remove(road_id);
        self.electricity.remove_object(road_id);

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
    pub(crate) fn split_road(
        &mut self,
        split_road_id: RoadID,
        pos: Vec3,
    ) -> Option<IntersectionID> {
        info!("split_road {:?} {:?}", split_road_id, pos);

        if !self.roads.contains_key(split_road_id) {
            log::error!("splitting non-existing road {:?}", split_road_id);
            return None;
        }

        let pat = self.roads.get(split_road_id)?.pattern(&self.lanes);
        let r = self.remove_raw_road(split_road_id)?;
        self.subscribers.dispatch(UpdateType::Road, &r);

        for (id, _) in r.lanes_iter() {
            self.parking.remove_to_reuse(id);
        }

        let id = self.add_intersection(pos - Vec3::z(ROAD_Z_OFFSET));

        let dist_along = r.points.length_at_proj(pos);
        let (before, after) = r.points.split(dist_along);

        let r1 = self.connect(r.src, id, &pat, RoadSegmentKind::Arbitrary(before))?;
        let r2 = self.connect(id, r.dst, &pat, RoadSegmentKind::Arbitrary(after))?;

        self.invalidate(r.src);
        self.invalidate(r.dst);
        self.invalidate(id);

        log::info!(
            "{} parking spots reused when splitting",
            self.parking.clean_reuse()
        );

        let r1 = self.roads.get(r1)?;
        let r2 = self.roads.get(r2)?;

        let spatial = &mut self.spatial_map;

        for b in &r.connected_buildings {
            let b = self.buildings.get_mut(*b)?;

            let bpos = b.obb.center().z(b.height);

            let d1 = r1.points.project_dist2(bpos);
            let d2 = r2.points.project_dist2(bpos);

            b.connected_road = Some(match d1 < d2 {
                true => r1.id,
                false => r2.id,
            });
        }

        self.lots.retain(|_, lot| {
            if lot.parent != split_road_id {
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

        for b in r.connected_buildings {
            let b = &self.buildings[b];
            let road_id = b.connected_road.unwrap();
            self.roads[road_id].connected_buildings.push(b.id);

            self.electricity.add_edge(b.id, road_id);
        }

        Some(id)
    }

    pub(crate) fn merge_road(&mut self, road1: RoadID, road2: RoadID) -> Option<RoadID> {
        info!("merge_road {:?} {:?}", road1, road2);

        let r1 = self.roads.get(road1)?;
        let r2 = self.roads.get(road2)?;

        let same_inter = if r1.src == r2.src || r1.src == r2.dst {
            r1.src
        } else if r1.dst == r2.src || r1.dst == r2.dst {
            r1.dst
        } else {
            warn!("trying to merge roads that are not connected to each other");
            return None;
        };

        let mut pat1 = r1.pattern(&self.lanes);
        let mut pat2 = r2.pattern(&self.lanes);

        if r1.src == same_inter {
            std::mem::swap(&mut pat1.lanes_backward, &mut pat1.lanes_forward);
        }

        if r2.src != same_inter {
            std::mem::swap(&mut pat2.lanes_backward, &mut pat2.lanes_forward);
        }

        if pat1 != pat2 {
            log::info!("merge refused because patterns don't match");
            return None;
        }

        let r1_extremity = if r1.src == same_inter { r1.dst } else { r1.src };
        let r2_extremity = if r2.src == same_inter { r2.dst } else { r2.src };

        if r1_extremity == r2_extremity {
            log::info!("merge refused because connecting to itself");
            return None;
        }

        let r1 = self.remove_raw_road(road1)?;
        let r2 = self.remove_raw_road(road2)?;

        self.remove_intersection_inner(same_inter);

        self.subscribers.dispatch(UpdateType::Road, &r1);
        self.subscribers.dispatch(UpdateType::Road, &r2);

        for (id, _) in r2.lanes_iter().chain(r1.lanes_iter()) {
            self.parking.remove_to_reuse(id);
        }

        let mut new_polyline = r1.points;
        if r1.src == same_inter {
            new_polyline.reverse();
        }

        if r2.src == same_inter {
            new_polyline.extend(r2.points.iter().skip(1))
        } else {
            new_polyline.extend(r2.points.iter().rev().skip(1))
        }

        let new_r = self.connect(
            r1_extremity,
            r2_extremity,
            &pat1,
            RoadSegmentKind::Arbitrary(new_polyline),
        )?;

        let new_r = &mut self.roads[new_r];
        log::info!("merge_road new road is {:?}", new_r.id);

        log::info!(
            "{} parking spots reused when merging",
            self.parking.clean_reuse()
        );

        for b in r1
            .connected_buildings
            .iter()
            .chain(r2.connected_buildings.iter())
        {
            let b = self.buildings.get_mut(*b)?;
            b.connected_road = Some(new_r.id);
            new_r.connected_buildings.push(b.id);

            self.electricity.add_edge(b.id, new_r.id);
        }

        for lot in &mut self.lots.values_mut() {
            if lot.parent == road1 || lot.parent == road2 {
                lot.parent = new_r.id;
            }
        }

        let new_id = new_r.id;

        self.invalidate(r1_extremity);
        self.invalidate(r2_extremity);

        Some(new_id)
    }

    /// Returns None if one of the intersections don't exist
    pub(crate) fn connect(
        &mut self,
        src_id: IntersectionID,
        dst_id: IntersectionID,
        pattern: &LanePattern,
        segment: RoadSegmentKind,
    ) -> Option<RoadID> {
        let src = self.intersections.get(src_id)?;
        let dst = self.intersections.get(dst_id)?;

        let gen_lots = !matches!(segment, RoadSegmentKind::Arbitrary(_));

        let rid = Road::make(
            src,
            dst,
            segment,
            pattern,
            &self.environment,
            &mut self.roads,
            &mut self.lanes,
            &mut self.parking,
            &mut self.spatial_map,
        );

        self.electricity.add_object(rid);
        self.electricity.add_edge(src_id, rid);
        self.electricity.add_edge(dst_id, rid);

        #[allow(clippy::indexing_slicing)]
        let r = &self.roads[rid];

        self.intersections.get_mut(src_id)?.add_road(&self.roads, r);
        self.intersections.get_mut(dst_id)?.add_road(&self.roads, r);

        Lot::remove_intersecting_lots(self, rid);
        if gen_lots {
            Lot::generate_along_road(self, rid);
        }

        #[allow(clippy::indexing_slicing)]
        let r = &self.roads[rid];
        let mut b = r.boldline();
        b.expand(40.0);
        self.environment.remove_trees_near(&b, |tree_chunk| {
            self.subscribers
                .dispatch_chunk(UpdateType::Terrain, tree_chunk)
        });

        Some(rid)
    }

    // Public helpers
    pub fn project(&self, pos: Vec3, tolerance: f32, filter: ProjectFilter) -> MapProject {
        let mk_proj = move |kind| MapProject { pos, kind };

        let mut qroad = None;
        for pkind in self.spatial_map.query_around(pos.xy(), tolerance, filter) {
            match pkind {
                ProjectKind::Intersection(id) => {
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

    pub fn get<T: MapObj>(&self, id: T) -> Option<&T::Obj> {
        MapObj::get(id, self)
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
            .project_segment_dir(spot.trans.pos);
        Some(pos - dir * 4.0)
    }

    #[cfg(not(debug_assertions))]
    pub fn check_invariants(&self) {}

    #[cfg(debug_assertions)]
    pub fn check_invariants(&self) {
        /*if std::env::var("MAP_INVARIANT_CHECK").is_err() {
            return;
        }*/
        for inter in self.intersections.values() {
            log::debug!("{:?}", inter.id);
            assert!(!inter.roads.is_empty());

            let mut last_angle = -f32::INFINITY;
            for &road in &inter.roads {
                let road = self.roads.get(road).expect("road does not exist");
                let ang = geom::pseudo_angle(road.dir_from(inter.id));
                assert!(ang >= last_angle);
                last_angle = ang;
            }

            for turn in inter.turns() {
                log::debug!("{:?}", turn.id);
                assert_eq!(turn.id.parent, inter.id);
                assert!(
                    self.lanes.contains_key(turn.id.src),
                    "Turn src doesn't exist for inter:{:?} src:{:?}",
                    inter.id,
                    turn.id.src,
                );
                assert!(
                    self.lanes.contains_key(turn.id.dst),
                    "{:?} {:?}",
                    inter.id,
                    turn.id.dst
                );
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

        crate::map::electricity_cache::check_electricity_coherency(self);

        for b in self.external_train_stations.iter() {
            assert!(self.buildings.contains_key(*b));
        }

        for road in self.roads.values() {
            log::debug!("{:?}", road.id);
            let src = self.intersections.get(road.src).unwrap();
            assert!(src.roads.contains(&road.id));
            let dst = self.intersections.get(road.dst).unwrap();
            assert!(dst.roads.contains(&road.id));
            assert!(!road.points.is_empty());
            assert!(road.lanes_iter().next().is_some());
            assert!(
                road.points
                    .first()
                    .up(-crate::map::ROAD_Z_OFFSET)
                    .is_close(src.pos, 0.001),
                "First road point should be close to src intersection's position {:?} {:?} {:?} {:?}",
                road.points.first().up(-crate::map::ROAD_Z_OFFSET),
                src.pos,
                road.id,
                src.id,
            );
            assert!(
                road.points
                    .last()
                    .up(-crate::map::ROAD_Z_OFFSET)
                    .is_close(dst.pos, 0.001),
                "Last road point should be close to dst intersection's position {:?} {:?} {:?} {:?}",
                road.points.last().up(-crate::map::ROAD_Z_OFFSET),
                dst.pos,
                road.id,
                dst.id,
            );
            assert!(road.interfaced_points().n_points() >= 2);
            assert!(road.length() > 0.0);
            assert!(self.spatial_map.contains(road.id));

            for (id, _) in road.lanes_iter() {
                let v = self.lanes.get(id).expect("lane child does not exist");
                assert_eq!(v.parent, road.id);
            }

            for b in &road.connected_buildings {
                assert!(self.buildings.contains_key(*b));
                assert_eq!(self.buildings[*b].connected_road, Some(road.id));
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

        for b in self.buildings.values() {
            log::debug!("{:?}", b.id);
            assert!(self.spatial_map.contains(b.id));
            if let Some(r) = b.connected_road {
                assert!(self.roads.contains_key(r));
                assert!(self.roads[r].connected_buildings.contains(&b.id));
            }
        }

        for lot in self.lots.values() {
            log::debug!("{:?}", lot.id);
            assert!(lot.shape.axis().iter().all(|x| x.mag() > 0.0));
            assert!(
                self.roads.contains_key(lot.parent),
                "Lot parent {:?} doesn't exist",
                lot.parent
            );
            assert!(self.spatial_map.contains(lot.id));
        }

        for obj in self.spatial_map.objects() {
            assert!(self.spatial_map.contains(*obj));
            log::debug!("{:?}", obj);
            assert!(obj.check_valid(self));
        }

        assert!(self.parking.reuse_spot.is_empty());

        log::info!("invariants checked");
    }
}

impl MapProject {
    pub fn ground(pos: Vec3) -> Self {
        Self {
            pos,
            kind: ProjectKind::Ground,
        }
    }

    pub fn is_ground(&self) -> bool {
        matches!(self.kind, ProjectKind::Ground)
    }
}

pub trait MapObj {
    type Obj;
    fn get(self, map: &Map) -> Option<&Self::Obj>;
}

impl MapObj for RoadID {
    type Obj = Road;
    fn get(self, map: &Map) -> Option<&Road> {
        map.roads.get(self)
    }
}

impl MapObj for LaneID {
    type Obj = Lane;
    fn get(self, map: &Map) -> Option<&Lane> {
        map.lanes.get(self)
    }
}

impl MapObj for IntersectionID {
    type Obj = Intersection;
    fn get(self, map: &Map) -> Option<&Intersection> {
        map.intersections.get(self)
    }
}

impl MapObj for BuildingID {
    type Obj = Building;
    fn get(self, map: &Map) -> Option<&Building> {
        map.buildings.get(self)
    }
}

impl MapObj for LotID {
    type Obj = Lot;
    fn get(self, map: &Map) -> Option<&Lot> {
        map.lots.get(self)
    }
}
