#![allow(clippy::or_fun_call)]
use crate::{LaneID, LaneKind, Map, Traversable, TraverseDirection, TraverseKind, TurnID};
use geom::{PolyLine, Vec2};
use ordered_float::OrderedFloat;
use slotmap::Key;

pub trait Pathfinder {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>>;
    fn nearest_lane(&self, map: &Map, pos: Vec2) -> Option<LaneID>;
    fn local_route(&self, map: &Map, lane: LaneID, start: Vec2, end: Vec2) -> Option<PolyLine>;
}

pub struct PedestrianPath;

impl Pathfinder for PedestrianPath {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>> {
        let inters = &map.intersections;
        let lanes = &map.lanes;

        let end_pos = inters[lanes[end].dst].pos;

        let heuristic = |t: &Traversable| {
            let pos = inters[t.destination_intersection(lanes)].pos;

            OrderedFloat(pos.distance(end_pos) * 1.3) // Inexact but (much) faster
        };

        let successors = |t: &Traversable| {
            let inter = &inters[t.destination_intersection(lanes)];
            let lane_from_id = t.destination_lane();
            let lane_from = &lanes[lane_from_id];
            let lane_travers = (
                Traversable::new(
                    TraverseKind::Lane(lane_from_id),
                    lane_from.dir_from(inter.id),
                ),
                OrderedFloat(lane_from.length),
            );

            inter
                .turns_from(lane_from_id)
                .map(|(x, dir)| {
                    (
                        Traversable::new(TraverseKind::Turn(x), dir),
                        OrderedFloat(0.001), // Unwrap ok: not nan
                    )
                })
                .chain(std::iter::once(lane_travers))
        };

        let has_arrived = |p: &Traversable| match p.kind {
            TraverseKind::Lane(id) => id == end,
            TraverseKind::Turn(_) => false,
        };

        pathfinding::directed::astar::astar(&start, successors, heuristic, has_arrived)
            .map(|(v, _)| v)
    }

    fn nearest_lane(&self, map: &Map, pos: Vec2) -> Option<LaneID> {
        map.nearest_lane(pos, LaneKind::Walking)
    }

    fn local_route(&self, map: &Map, lane: LaneID, start: Vec2, end: Vec2) -> Option<PolyLine> {
        let lane = &map.lanes[lane];
        let (p_start, seg_start) = lane.points.project_segment(start);
        let (p_end, seg_end) = lane.points.project_segment(end);

        let segs = &lane.points[seg_start.min(seg_end)..seg_start.max(seg_end)];
        let mut v = Vec::with_capacity(3 + segs.len());
        v.push(p_start);
        v.extend_from_slice(segs);
        v.push(p_end);
        v.push(end);
        Some(PolyLine::new(v))
    }
}

pub struct CarPath;

impl Pathfinder for CarPath {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>> {
        let inters = &map.intersections;
        let lanes = &map.lanes;

        let start_lane = start.destination_lane();

        let end_pos = inters[lanes[end].dst].pos;

        let dummy = LaneID::null();

        let heuristic = |&p: &LaneID| {
            let pos = inters[lanes[p].dst].pos;
            OrderedFloat(pos.distance(end_pos) * 1.2) // Inexact but (much) faster
        };

        let successors = |&p: &LaneID| {
            let l;
            let p = if p == dummy {
                l = &lanes[start_lane];
                start_lane
            } else {
                l = &lanes[p];
                p
            };
            let inter = &inters[l.dst];
            inter
                .turns_from(p)
                .map(|(x, _)| (x.dst, OrderedFloat(lanes[x.dst].length)))
        };

        let (v, _) =
            pathfinding::directed::astar::astar(&dummy, successors, heuristic, |p| *p == end)?;

        let mut path = Vec::with_capacity(v.len() * 2);
        path.push(start);

        let mut last_id = start_lane;

        for lane in v.into_iter().skip(1) {
            let inter_end = &inters[lanes[lane].src];
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

    fn nearest_lane(&self, map: &Map, pos: Vec2) -> Option<LaneID> {
        map.nearest_lane(pos, LaneKind::Driving)
    }

    fn local_route(&self, map: &Map, lane: LaneID, start: Vec2, end: Vec2) -> Option<PolyLine> {
        let lane = &map.lanes[lane];
        let (p_start, seg_start) = lane.points.project_segment(start);
        let (p_end, seg_end) = lane.points.project_segment(end);

        if seg_end < seg_start
            || (seg_end == seg_start
                && lane.points[seg_end].distance2(p_start) < lane.points[seg_end].distance2(p_end))
        {
            return None;
        }

        let segs = &lane.points[seg_start..seg_end];
        let mut v = Vec::with_capacity(3 + segs.len());
        v.push(p_start);
        v.extend_from_slice(segs);
        v.push(p_end);
        v.push(end);
        Some(PolyLine::new(v))
    }
}
