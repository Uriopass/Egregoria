use crate::map_model::{Intersection, IntersectionID, LaneID, Lanes, Roads, TurnID, TurnKind};
use cgmath::InnerSpace;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use std::iter::{Extend, Iterator};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Inspect)]
pub struct TurnPolicy {
    pub back_turns: bool,
    pub left_turns: bool,
}

impl Default for TurnPolicy {
    fn default() -> Self {
        Self {
            back_turns: false,
            left_turns: true,
        }
    }
}

fn filter_vehicles(x: &[LaneID], lanes: &Lanes) -> Vec<LaneID> {
    x.iter()
        .filter(|x| lanes[**x].kind.vehicles())
        .copied()
        .collect::<Vec<_>>()
}

impl TurnPolicy {
    fn zip(
        inter_id: IntersectionID,
        incoming: &[LaneID],
        outgoing: &[LaneID],
    ) -> Vec<(TurnID, TurnKind)> {
        incoming
            .iter()
            .zip(outgoing)
            .map(|(lane_src, lane_dst)| {
                (
                    TurnID::new(inter_id, *lane_src, *lane_dst, false),
                    TurnKind::Driving,
                )
            })
            .collect()
    }

    fn all(
        inter_id: IntersectionID,
        incoming: &[LaneID],
        outgoing: &[LaneID],
    ) -> Vec<(TurnID, TurnKind)> {
        incoming
            .iter()
            .map(|lane_src| {
                outgoing.iter().map(move |lane_dst| {
                    (
                        TurnID::new(inter_id, *lane_src, *lane_dst, false),
                        TurnKind::Driving,
                    )
                })
            })
            .flatten()
            .collect()
    }

    fn zip_on_same_length(
        inter_id: IntersectionID,
        incoming: &[LaneID],
        outgoing: &[LaneID],
    ) -> Vec<(TurnID, TurnKind)> {
        if incoming.len() == outgoing.len() {
            Self::zip(inter_id, incoming, outgoing)
        } else {
            Self::all(inter_id, incoming, outgoing)
        }
    }

    pub fn generate_vehicle_turns(
        self,
        inter: &Intersection,
        lanes: &Lanes,
        roads: &Roads,
        turns: &mut Vec<(TurnID, TurnKind)>,
    ) {
        match inter.roads.as_slice() {
            [road_id] => {
                let road = &roads[*road_id];
                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &filter_vehicles(road.incoming_lanes_to(inter.id), lanes),
                    &filter_vehicles(road.outgoing_lanes_from(inter.id), lanes),
                ));
                return;
            }
            [road1, road2] => {
                let road1 = &roads[*road1];
                let road2 = &roads[*road2];

                let incoming_road1 = filter_vehicles(road1.incoming_lanes_to(inter.id), lanes);
                let incoming_road2 = filter_vehicles(road2.incoming_lanes_to(inter.id), lanes);

                let outgoing_road1 = filter_vehicles(road1.outgoing_lanes_from(inter.id), lanes);
                let outgoing_road2 = filter_vehicles(road2.outgoing_lanes_from(inter.id), lanes);

                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &incoming_road1,
                    &outgoing_road2,
                ));

                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &incoming_road2,
                    &outgoing_road1,
                ));

                return;
            }
            _ => {}
        }

        for road1 in &inter.roads {
            for road2 in &inter.roads {
                if road1 == road2 && !self.back_turns {
                    continue;
                }

                for incoming in roads[*road1].incoming_lanes_to(inter.id) {
                    for outgoing in roads[*road2].outgoing_lanes_from(inter.id) {
                        let incoming = &lanes[*incoming];
                        let outgoing = &lanes[*outgoing];
                        if !incoming.kind.vehicles() || !outgoing.kind.vehicles() {
                            continue;
                        }

                        let incoming_dir = incoming.orientation_from(inter.id);
                        let outgoing_dir = outgoing.orientation_from(inter.id);

                        let incoming_right = vec2!(incoming_dir.y, -incoming_dir.x);
                        let id = TurnID::new(inter.id, incoming.id, outgoing.id, false);

                        if self.left_turns || incoming_right.dot(outgoing_dir) >= -0.3 {
                            turns.push((id, TurnKind::Driving));
                        }
                    }
                }
            }
        }
    }

    pub fn generate_walking_turns(
        self,
        inter: &Intersection,
        lanes: &Lanes,
        roads: &Roads,
        turns: &mut Vec<(TurnID, TurnKind)>,
    ) {
        let n_roads = inter.roads.len();

        for w in inter
            .roads
            .iter()
            .chain(inter.roads.iter().take(1))
            .map(|x| roads[*x].sidewalks(inter.id, lanes))
            .collect::<Vec<_>>()
            .windows(2)
        {
            if let [(incoming, outgoing_in), (_, outgoing)] = *w {
                if let (Some(incoming), Some(outgoing)) = (incoming, outgoing) {
                    turns.push((
                        TurnID::new(inter.id, incoming.id, outgoing.id, true),
                        TurnKind::WalkingCorner,
                    ));
                }

                if let (Some(incoming), Some(outgoing_in)) = (incoming, outgoing_in) {
                    if n_roads > 2 {
                        turns.push((
                            TurnID::new(inter.id, incoming.id, outgoing_in.id, true),
                            TurnKind::Crosswalk,
                        ));
                    }
                }
            }
        }
    }

    pub fn generate_turns(
        self,
        inter: &Intersection,
        lanes: &Lanes,
        roads: &Roads,
    ) -> Vec<(TurnID, TurnKind)> {
        let mut turns = vec![];

        self.generate_vehicle_turns(inter, lanes, roads, &mut turns);

        self.generate_walking_turns(inter, lanes, roads, &mut turns);

        turns
    }
}
