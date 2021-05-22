use crate::{LaneID, LaneKind, Map, Traversable, TraverseDirection, TraverseKind, TurnID};
use geom::{PolyLine3, Vec3};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::Key;

pub trait Pathfinder {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>>;
    fn nearest_lane(&self, map: &Map, pos: Vec3) -> Option<LaneID>;
    fn local_route(&self, map: &Map, lane: LaneID, start: Vec3, end: Vec3) -> Option<PolyLine3>;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PathKind {
    Pedestrian,
    Vehicle,
}

impl Pathfinder for PathKind {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>> {
        match self {
            PathKind::Pedestrian => PedestrianPath.path(map, start, end),
            PathKind::Vehicle => CarPath.path(map, start, end),
        }
    }

    fn nearest_lane(&self, map: &Map, pos: Vec3) -> Option<LaneID> {
        match self {
            PathKind::Pedestrian => PedestrianPath.nearest_lane(map, pos),
            PathKind::Vehicle => CarPath.nearest_lane(map, pos),
        }
    }

    fn local_route(&self, map: &Map, lane: LaneID, start: Vec3, end: Vec3) -> Option<PolyLine3> {
        match self {
            PathKind::Pedestrian => PedestrianPath.local_route(map, lane, start, end),
            PathKind::Vehicle => CarPath.local_route(map, lane, start, end),
        }
    }
}

struct PedestrianPath;

impl Pathfinder for PedestrianPath {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>> {
        let inters = &map.intersections;
        let lanes = &map.lanes;

        let end_pos = inters.get(lanes.get(end)?.dst)?.pos;

        let heuristic = |t: &Traversable| {
            let pos = unwrap_ret!(
                inters.get(unwrap_ret!(
                    t.destination_intersection(lanes),
                    OrderedFloat(f32::INFINITY)
                )),
                OrderedFloat(f32::INFINITY)
            )
            .pos;

            OrderedFloat(pos.distance(end_pos) * 1.3) // Inexact but (much) faster
        };

        let successors = |t: &Traversable| {
            let inter = t
                .destination_intersection(lanes)
                .and_then(|x| inters.get(x));
            let lane_from_id = t.destination_lane();
            let lane_from = lanes.get(lane_from_id);

            let lane_travers = inter.zip(lane_from).map(|(inter, lane_from)| {
                (
                    Traversable::new(
                        TraverseKind::Lane(lane_from_id),
                        lane_from.dir_from(inter.id),
                    ),
                    OrderedFloat(lane_from.points.length()),
                )
            });

            inter
                .into_iter()
                .flat_map(move |inter| {
                    inter.turns_from(lane_from_id).map(|(x, dir)| {
                        (
                            Traversable::new(TraverseKind::Turn(x), dir),
                            OrderedFloat(0.001),
                        )
                    })
                })
                .chain(lane_travers)
        };

        let has_arrived = |p: &Traversable| match p.kind {
            TraverseKind::Lane(id) => id == end,
            TraverseKind::Turn(_) => false,
        };

        pathfinding::directed::astar::astar(&start, successors, heuristic, has_arrived)
            .map(|(v, _)| v)
    }

    fn nearest_lane(&self, map: &Map, pos: Vec3) -> Option<LaneID> {
        map.nearest_lane(pos, LaneKind::Walking)
    }

    fn local_route(&self, map: &Map, lane: LaneID, start: Vec3, end: Vec3) -> Option<PolyLine3> {
        let lane = map.lanes.get(lane)?;
        let (p_start, seg_start) = lane.points.project_segment(start);
        let (p_end, seg_end) = lane.points.project_segment(end);

        let segs = lane
            .points
            .get(seg_start.min(seg_end)..seg_start.max(seg_end))?;
        let mut v = Vec::with_capacity(3 + segs.len());
        v.push(p_start);
        v.extend_from_slice(segs);
        v.push(p_end);
        v.push(end);
        Some(PolyLine3::new(v))
    }
}

struct CarPath;

impl Pathfinder for CarPath {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>> {
        let inters = &map.intersections;
        let lanes = &map.lanes;

        let start_lane = start.destination_lane();

        let end_pos = inters.get(lanes.get(end)?.dst)?.pos;

        let dummy = LaneID::null();

        let heuristic = |&p: &LaneID| {
            let pos = unwrap_ret!(
                inters.get(unwrap_ret!(lanes.get(p), OrderedFloat(f32::INFINITY)).dst),
                OrderedFloat(f32::INFINITY)
            )
            .pos;
            OrderedFloat(pos.distance(end_pos) * 1.2) // Inexact but (much) faster
        };

        let successors = |&p: &LaneID| {
            let l;
            let p = if p == dummy {
                l = lanes.get(start_lane);
                start_lane
            } else {
                l = lanes.get(p);
                p
            };
            l.and_then(|x| inters.get(x.dst))
                .into_iter()
                .flat_map(move |inter| {
                    inter.turns_from(p).map(move |(x, _)| {
                        (
                            x.dst,
                            OrderedFloat(
                                lanes
                                    .get(x.dst)
                                    .map(|p| p.points.length())
                                    .unwrap_or(f32::INFINITY),
                            ),
                        )
                    })
                })
        };

        let (v, _) =
            pathfinding::directed::astar::astar(&dummy, successors, heuristic, |p| *p == end)?;

        let mut path = Vec::with_capacity(v.len() * 2);
        path.push(start);

        let mut last_id = start_lane;

        for lane in v.into_iter().skip(1) {
            let inter_end = &inters.get(lanes.get(lane)?.src)?;
            let id = TurnID::new(inter_end.id, last_id, lane, false);
            path.push(Traversable::new(
                TraverseKind::Turn(id),
                TraverseDirection::Forward,
            ));
            path.push(Traversable::new(
                TraverseKind::Lane(lane),
                TraverseDirection::Forward,
            ));

            last_id = lane;
        }
        Some(path)
    }

    fn nearest_lane(&self, map: &Map, pos: Vec3) -> Option<LaneID> {
        map.nearest_lane(pos, LaneKind::Driving)
    }

    fn local_route(&self, map: &Map, lane: LaneID, start: Vec3, end: Vec3) -> Option<PolyLine3> {
        let lane = &map.lanes.get(lane)?;
        let (p_start, seg_start) = lane.points.project_segment(start);
        let (p_end, seg_end) = lane.points.project_segment(end);

        if seg_end < seg_start
            || (seg_end == seg_start
                && lane.points.get(seg_end)?.distance2(p_start)
                    < lane.points.get(seg_end)?.distance2(p_end))
        {
            return None;
        }

        let segs = &lane.points.get(seg_start..seg_end)?;
        let mut v = Vec::with_capacity(3 + segs.len());
        v.push(p_start);
        v.extend_from_slice(segs);
        v.push(p_end);
        v.push(end);
        Some(PolyLine3::new(v))
    }
}
