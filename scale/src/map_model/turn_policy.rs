use crate::map_model::{Intersection, IntersectionID, LaneID, Lanes, Roads, TurnID};
use cgmath::{vec2, InnerSpace};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use std::iter::{Extend, Iterator};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Inspect)]
pub struct TurnPolicy {
    back_turns: bool,
    left_turns: bool,
}

impl Default for TurnPolicy {
    fn default() -> Self {
        Self {
            back_turns: false,
            left_turns: true,
        }
    }
}

impl TurnPolicy {
    fn zip(inter_id: IntersectionID, incoming: &[LaneID], outgoing: &[LaneID]) -> Vec<TurnID> {
        incoming
            .iter()
            .zip(outgoing)
            .map(|(lane_src, lane_dst)| TurnID::new(inter_id, *lane_src, *lane_dst))
            .collect()
    }

    fn all(inter_id: IntersectionID, incoming: &[LaneID], outgoing: &[LaneID]) -> Vec<TurnID> {
        incoming
            .iter()
            .map(|lane_src| {
                outgoing
                    .iter()
                    .map(move |lane_dst| TurnID::new(inter_id, *lane_src, *lane_dst))
            })
            .flatten()
            .collect()
    }

    fn zip_on_same_length(
        inter_id: IntersectionID,
        incoming: &[LaneID],
        outgoing: &[LaneID],
    ) -> Vec<TurnID> {
        if incoming.len() == outgoing.len() {
            Self::zip(inter_id, incoming, outgoing)
        } else {
            Self::all(inter_id, incoming, outgoing)
        }
    }

    pub fn generate_turns(self, inter: &Intersection, lanes: &Lanes, roads: &Roads) -> Vec<TurnID> {
        if let [road_id] = inter.roads.as_slice() {
            let road = &roads[*road_id];
            return Self::zip_on_same_length(
                inter.id,
                road.incoming_lanes_from(inter.id),
                road.outgoing_lanes_from(inter.id),
            );
        }

        let mut turns = vec![];

        if let [road1, road2] = inter.roads.as_slice() {
            let road1 = &roads[*road1];
            let road2 = &roads[*road2];

            let incoming_road1 = road1.incoming_lanes_from(inter.id);
            let incoming_road2 = road2.incoming_lanes_from(inter.id);

            let outgoing_road1 = road1.outgoing_lanes_from(inter.id);
            let outgoing_road2 = road2.outgoing_lanes_from(inter.id);

            turns.extend(Self::zip_on_same_length(
                inter.id,
                incoming_road1,
                outgoing_road2,
            ));

            turns.extend(Self::zip_on_same_length(
                inter.id,
                incoming_road2,
                outgoing_road1,
            ));

            return turns;
        }

        for road1 in &inter.roads {
            for road2 in &inter.roads {
                if road1 == road2 && !self.back_turns {
                    continue;
                }

                for incoming in roads[*road1].incoming_lanes_from(inter.id) {
                    for outgoing in roads[*road2].outgoing_lanes_from(inter.id) {
                        let incoming_dir = lanes[*incoming].get_orientation_vec();
                        let outgoing_dir = lanes[*outgoing].get_orientation_vec();

                        let incoming_right = vec2(incoming_dir.y, -incoming_dir.x);
                        let id = TurnID::new(inter.id, *incoming, *outgoing);

                        if self.left_turns || incoming_right.dot(outgoing_dir) >= -0.3 {
                            turns.push(id);
                        }
                    }
                }
            }
        }

        turns
    }
}
