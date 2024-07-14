use crate::map::{Intersection, IntersectionID, LaneID, LaneKind, Lanes, Roads, TurnID, TurnKind};
use egui_inspect::{Inspect, OptionDefault};
use geom::{vec2, Vec2};
use serde::{Deserialize, Serialize};
use std::iter::{Extend, Iterator};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Inspect)]
pub struct RoundaboutPolicy {
    #[inspect(min_value = 10.0, max_value = 50.0)]
    pub radius: f32,
}

impl Default for RoundaboutPolicy {
    fn default() -> Self {
        Self { radius: 20.0 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Inspect)]
pub struct TurnPolicy {
    pub back_turns: bool,
    pub left_turns: bool,
    pub crosswalks: bool,
    #[inspect(proxy_type = "OptionDefault")]
    pub roundabout: Option<RoundaboutPolicy>,
}

impl Default for TurnPolicy {
    fn default() -> Self {
        Self {
            back_turns: false,
            left_turns: true,
            crosswalks: true,
            roundabout: None,
        }
    }
}

fn filter_vehicles(x: &[(LaneID, LaneKind)]) -> Vec<LaneID> {
    x.iter()
        .filter(|(_, kind)| kind.vehicles())
        .map(|(id, _)| id)
        .copied()
        .collect::<Vec<_>>()
}

fn filter_rail(x: &[(LaneID, LaneKind)]) -> Vec<LaneID> {
    x.iter()
        .filter(|(_, kind)| kind.is_rail())
        .map(|(id, _)| id)
        .copied()
        .collect::<Vec<_>>()
}

impl TurnPolicy {
    fn zip(
        inter_id: IntersectionID,
        incoming: &[LaneID],
        outgoing: &[LaneID],
        turnkind: TurnKind,
    ) -> Vec<(TurnID, TurnKind)> {
        incoming
            .iter()
            .zip(outgoing)
            .map(|(lane_src, lane_dst)| {
                (TurnID::new(inter_id, *lane_src, *lane_dst, false), turnkind)
            })
            .collect()
    }

    fn all(
        inter_id: IntersectionID,
        incoming: &[LaneID],
        outgoing: &[LaneID],
        turnkind: TurnKind,
    ) -> Vec<(TurnID, TurnKind)> {
        incoming
            .iter()
            .flat_map(|lane_src| {
                outgoing.iter().map(move |lane_dst| {
                    (TurnID::new(inter_id, *lane_src, *lane_dst, false), turnkind)
                })
            })
            .collect()
    }

    fn zip_on_same_length(
        inter_id: IntersectionID,
        incoming: &[LaneID],
        outgoing: &[LaneID],
        turnkind: TurnKind,
    ) -> Vec<(TurnID, TurnKind)> {
        if incoming.len() == outgoing.len() {
            Self::zip(inter_id, incoming, outgoing, turnkind)
        } else {
            Self::all(inter_id, incoming, outgoing, turnkind)
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
                let road = unwrap_ret!(roads.get(*road_id));
                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &filter_vehicles(road.incoming_lanes_to(inter.id)),
                    &filter_vehicles(road.outgoing_lanes_from(inter.id)),
                    TurnKind::Driving,
                ));
                return;
            }
            [road1, road2] => {
                let road1 = unwrap_ret!(roads.get(*road1));
                let road2 = unwrap_ret!(roads.get(*road2));

                let incoming_road1 = filter_vehicles(road1.incoming_lanes_to(inter.id));
                let incoming_road2 = filter_vehicles(road2.incoming_lanes_to(inter.id));

                let outgoing_road1 = filter_vehicles(road1.outgoing_lanes_from(inter.id));
                let outgoing_road2 = filter_vehicles(road2.outgoing_lanes_from(inter.id));

                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &incoming_road1,
                    &outgoing_road2,
                    TurnKind::Driving,
                ));

                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &incoming_road2,
                    &outgoing_road1,
                    TurnKind::Driving,
                ));

                if self.back_turns {
                    turns.extend(Self::zip_on_same_length(
                        inter.id,
                        &incoming_road1,
                        &outgoing_road1,
                        TurnKind::Driving,
                    ));

                    turns.extend(Self::zip_on_same_length(
                        inter.id,
                        &incoming_road2,
                        &outgoing_road2,
                        TurnKind::Driving,
                    ));
                }

                return;
            }
            _ => {}
        }

        let n_roads = inter.roads.len();

        for (i1, road1) in inter.roads.iter().enumerate() {
            for (i2, road2) in inter.roads.iter().enumerate() {
                if road1 == road2 && !self.back_turns {
                    continue;
                }

                let r1 = unwrap_cont!(roads.get(*road1));
                let r2 = unwrap_cont!(roads.get(*road2));
                for (incoming, incoming_kind) in r1.incoming_lanes_to(inter.id) {
                    for (outgoing, outgoing_kind) in r2.outgoing_lanes_from(inter.id) {
                        if !incoming_kind.vehicles() || !outgoing_kind.vehicles() {
                            continue;
                        }

                        let incoming = unwrap_cont!(lanes.get(*incoming));
                        let outgoing = unwrap_cont!(lanes.get(*outgoing));

                        let incoming_dir = incoming.orientation_from(inter.id);
                        let outgoing_dir = outgoing.orientation_from(inter.id);

                        let incoming_right = vec2(incoming_dir.y, -incoming_dir.x);
                        let id = TurnID::new(inter.id, incoming.id, outgoing.id, false);

                        if self.left_turns
                            || incoming_right.dot(outgoing_dir) <= 0.1
                            || i2 == (i1 + 1) % n_roads
                        {
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
        roads: &Roads,
        turns: &mut Vec<(TurnID, TurnKind)>,
    ) {
        let n_roads = inter.roads.len();

        for w in inter
            .roads
            .iter()
            .chain(inter.roads.iter().take(1))
            .flat_map(|x| Some(roads.get(*x)?.sidewalks(inter.id)))
            .collect::<Vec<_>>()
            .windows(2)
        {
            if let [a, b] = *w {
                if let (Some(incoming), Some(outgoing)) = (a.incoming, b.outgoing) {
                    turns.push((
                        TurnID::new(inter.id, incoming, outgoing, true),
                        TurnKind::WalkingCorner,
                    ));
                }

                if self.crosswalks && n_roads > 2 {
                    if let (Some(incoming), Some(outgoing_in)) = (a.incoming, a.outgoing) {
                        turns.push((
                            TurnID::new(inter.id, incoming, outgoing_in, true),
                            TurnKind::Crosswalk,
                        ));
                    }
                }
            }
        }
    }

    pub fn compatible_turn_sharpness_rail(dir1: Vec2, dir2: Vec2) -> bool {
        dir1.dot(dir2) <= -0.2
    }

    pub fn generate_rail_turns(
        self,
        inter: &Intersection,
        lanes: &Lanes,
        roads: &Roads,
        turns: &mut Vec<(TurnID, TurnKind)>,
    ) {
        match inter.roads.as_slice() {
            [road_id] => {
                let road = unwrap_ret!(roads.get(*road_id));
                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &filter_rail(road.incoming_lanes_to(inter.id)),
                    &filter_rail(road.outgoing_lanes_from(inter.id)),
                    TurnKind::Rail,
                ));
                return;
            }
            [road1, road2] => {
                let road1 = unwrap_ret!(roads.get(*road1));
                let road2 = unwrap_ret!(roads.get(*road2));

                let incoming_road1 = filter_rail(road1.incoming_lanes_to(inter.id));
                let incoming_road2 = filter_rail(road2.incoming_lanes_to(inter.id));

                let outgoing_road1 = filter_rail(road1.outgoing_lanes_from(inter.id));
                let outgoing_road2 = filter_rail(road2.outgoing_lanes_from(inter.id));

                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &incoming_road1,
                    &outgoing_road2,
                    TurnKind::Rail,
                ));

                turns.extend(Self::zip_on_same_length(
                    inter.id,
                    &incoming_road2,
                    &outgoing_road1,
                    TurnKind::Rail,
                ));

                return;
            }
            _ => {}
        }

        for road1 in &inter.roads {
            for road2 in &inter.roads {
                if road1 == road2 {
                    continue;
                }

                let r1 = unwrap_cont!(roads.get(*road1));
                let r2 = unwrap_cont!(roads.get(*road2));
                for (incoming, incoming_kind) in r1.incoming_lanes_to(inter.id) {
                    for (outgoing, outgoing_kind) in r2.outgoing_lanes_from(inter.id) {
                        if !incoming_kind.is_rail() || !outgoing_kind.is_rail() {
                            continue;
                        }

                        let incoming = unwrap_cont!(lanes.get(*incoming));
                        let outgoing = unwrap_cont!(lanes.get(*outgoing));

                        let incoming_dir = incoming.orientation_from(inter.id);
                        let outgoing_dir = outgoing.orientation_from(inter.id);

                        if Self::compatible_turn_sharpness_rail(incoming_dir, outgoing_dir) {
                            let id = TurnID::new(inter.id, incoming.id, outgoing.id, false);
                            turns.push((id, TurnKind::Rail));
                        }
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
        self.generate_rail_turns(inter, lanes, roads, &mut turns);

        self.generate_walking_turns(inter, roads, &mut turns);

        turns
    }
}
