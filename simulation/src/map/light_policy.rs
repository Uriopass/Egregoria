use crate::map::{Intersection, LaneID, Lanes, Roads, TrafficControl, TrafficLightSchedule};
use egui_inspect::{egui, egui::Ui, Inspect, InspectArgs};
use prototypes::SECONDS_PER_REALTIME_SECOND;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightPolicy {
    NoLights,
    StopSigns,
    Lights,
    #[default]
    Auto,
}

impl LightPolicy {
    pub fn apply(self, inter: &Intersection, lanes: &mut Lanes, roads: &Roads) {
        let in_road_lanes: Vec<Vec<LaneID>> = inter
            .roads
            .iter()
            .map(|&x| {
                roads
                    .get(x)
                    .into_iter()
                    .flat_map(|r| {
                        r.incoming_lanes_to(inter.id)
                            .iter()
                            .filter(|(_, kind)| kind.needs_light())
                            .map(|&(id, _)| id)
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|v| !v.is_empty())
            .collect();

        for incoming_lanes in &in_road_lanes {
            for &lane in incoming_lanes {
                unwrap_cont!(lanes.get_mut(lane)).control = TrafficControl::Always;
            }
        }

        match self {
            LightPolicy::NoLights => {}
            LightPolicy::StopSigns => {
                Self::stop_signs(in_road_lanes, lanes);
            }
            LightPolicy::Lights => {
                Self::lights(in_road_lanes, inter, lanes);
            }
            LightPolicy::Auto => {
                if in_road_lanes.len() <= 2 {
                    return;
                }
                if in_road_lanes.len() == 3 {
                    Self::stop_signs(in_road_lanes, lanes);
                    return;
                }

                if inter.turn_policy.left_turns {
                    Self::lights(in_road_lanes, inter, lanes);
                } else {
                    Self::stop_signs(in_road_lanes, lanes);
                }
            }
        }
    }

    pub fn is_stop_signs(&self) -> bool {
        matches!(self, LightPolicy::StopSigns)
    }

    fn stop_signs(in_road_lanes: Vec<Vec<LaneID>>, lanes: &mut Lanes) {
        for incoming_lanes in in_road_lanes {
            for lane in incoming_lanes {
                unwrap_cont!(lanes.get_mut(lane)).control = TrafficControl::StopSign;
            }
        }
    }

    fn lights(in_road_lanes: Vec<Vec<LaneID>>, inter: &Intersection, lanes: &mut Lanes) {
        let n_cycles = ((in_road_lanes.len() + 1) / 2) as u16;
        let cycle_size = 14 * SECONDS_PER_REALTIME_SECOND as u16;
        let orange_length = 4 * SECONDS_PER_REALTIME_SECOND as u16;

        let total_length = cycle_size * n_cycles;

        let inter_offset =
            (common::rand::rand(inter.id.as_ffi() as f32) * total_length as f32) as u16;

        for (i, incoming_lanes) in in_road_lanes.into_iter().enumerate() {
            let i = i as u16;
            let light = TrafficControl::Light(TrafficLightSchedule::from_basic(
                cycle_size - orange_length,
                orange_length,
                total_length - cycle_size,
                cycle_size * (i % n_cycles) + inter_offset,
            ));

            for lane in incoming_lanes {
                unwrap_cont!(lanes.get_mut(lane)).control = light;
            }
        }
    }
}

impl Inspect<LightPolicy> for LightPolicy {
    fn render(_: &LightPolicy, _: &'static str, _: &mut Ui, _: &InspectArgs) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut LightPolicy,
        label: &'static str,
        ui: &mut Ui,
        _: &InspectArgs,
    ) -> bool {
        let p = data;
        let mut id = match p {
            LightPolicy::NoLights => 0,
            LightPolicy::StopSigns => 1,
            LightPolicy::Lights => 2,
            LightPolicy::Auto => 3,
        };

        let tostr = |x: LightPolicy| match x {
            LightPolicy::NoLights => "No lights",
            LightPolicy::StopSigns => "Stop signs",
            LightPolicy::Lights => "Lights",
            LightPolicy::Auto => "Auto",
        };

        let get = |i| match i {
            0 => LightPolicy::NoLights,
            1 => LightPolicy::StopSigns,
            2 => LightPolicy::Lights,
            3 => LightPolicy::Auto,
            _ => unreachable!(),
        };

        let changed = egui::ComboBox::from_label(label)
            .show_index(ui, &mut id, 4, |i| tostr(get(i)).to_string())
            .changed();
        if changed {
            *p = get(id);
        }

        changed
    }
}
