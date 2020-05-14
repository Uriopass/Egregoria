use crate::map_model::{LaneID, Map, Traversable, TraverseDirection, TraverseKind, TurnID};
use cgmath::{MetricSpace, Zero};
use ordered_float::NotNan;

pub trait Pathfinder {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>>;
}

pub struct PedestrianPath;

impl Pathfinder for PedestrianPath {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>> {
        let inters = map.intersections();
        let lanes = map.lanes();

        let end_pos = inters[lanes[end].dst].pos;

        let heuristic = |t: &Traversable| {
            let pos = inters[t.destination_intersection(lanes)].pos;

            NotNan::new(pos.distance(end_pos) * 1.3).unwrap() // Inexact but (much) faster
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
                NotNan::new(lane_from.inter_length).unwrap_or(NotNan::zero()),
            );

            inter
                .turns_from(lane_from_id)
                .map(|(x, dir)| {
                    (
                        Traversable::new(TraverseKind::Turn(x), dir),
                        NotNan::new(0.001).unwrap(),
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
}

pub struct DirectionalPath;

impl Pathfinder for DirectionalPath {
    fn path(&self, map: &Map, start: Traversable, end: LaneID) -> Option<Vec<Traversable>> {
        let inters = map.intersections();
        let lanes = map.lanes();

        let start_lane = start.destination_lane();

        let end_pos = inters[lanes[end].dst].pos;

        let heuristic = |p: &LaneID| {
            let pos = inters[lanes[*p].dst].pos;
            NotNan::new(pos.distance(end_pos) * 1.2).unwrap() // Inexact but (much) faster
        };

        let successors = |p: &LaneID| {
            let l = &lanes[*p];
            let inter = &inters[l.dst];
            inter.turns_from(*p).map(|(x, _)| {
                (
                    x.dst,
                    NotNan::new(lanes[x.dst].inter_length).unwrap_or(NotNan::zero()),
                )
            })
        };

        let (v, _) =
            pathfinding::directed::astar::astar(&start_lane, successors, heuristic, |p| *p == end)?;

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
}
