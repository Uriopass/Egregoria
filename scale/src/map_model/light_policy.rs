use crate::map_model::{Intersection, LaneID, Lanes, Roads, TrafficControl, TrafficLightSchedule};
use imgui::{im_str, Ui};
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use specs::World;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LightPolicy {
    NoLights,
    StopSigns,
    Lights,
    Smart,
}

impl Default for LightPolicy {
    fn default() -> Self {
        LightPolicy::Smart
    }
}

impl LightPolicy {
    pub fn apply(self, inter: &Intersection, lanes: &mut Lanes, roads: &Roads) {
        let in_road_lanes: Vec<Vec<&LaneID>> = inter
            .roads
            .iter()
            .map(|&x| {
                roads[x]
                    .incoming_lanes_to(inter.id)
                    .iter()
                    .filter(|&&x| lanes[x].kind.needs_light())
                    .collect::<Vec<_>>()
            })
            .filter(|v| !v.is_empty())
            .collect();

        for incoming_lanes in &in_road_lanes {
            for &&lane in incoming_lanes {
                lanes[lane].control = TrafficControl::Always;
            }
        }

        match self {
            LightPolicy::NoLights => {}
            LightPolicy::StopSigns => {
                self.stop_signs(in_road_lanes, lanes);
            }
            LightPolicy::Lights => {
                self.lights(in_road_lanes, inter, lanes);
            }
            LightPolicy::Smart => {
                if in_road_lanes.len() <= 2 {
                    return;
                }

                if inter.turn_policy.left_turns {
                    self.lights(in_road_lanes, inter, lanes);
                } else {
                    self.stop_signs(in_road_lanes, lanes);
                }
            }
        }
    }

    fn stop_signs(self, in_road_lanes: Vec<Vec<&LaneID>>, lanes: &mut Lanes) {
        for incoming_lanes in in_road_lanes {
            for &lane in incoming_lanes {
                lanes[lane].control = TrafficControl::StopSign;
            }
        }
    }

    fn lights(self, in_road_lanes: Vec<Vec<&LaneID>>, inter: &Intersection, lanes: &mut Lanes) {
        let n_cycles = (in_road_lanes.len() + 1) / 2;
        let cycle_size = 14;
        let orange_length = 4;

        let total_length = cycle_size * n_cycles;

        let offset = inter.id.as_ffi();
        let inter_offset: usize =
            rand::rngs::SmallRng::seed_from_u64(offset as u64).gen_range(0, total_length);

        for (i, incoming_lanes) in in_road_lanes.into_iter().enumerate() {
            let light = TrafficControl::Light(TrafficLightSchedule::from_basic(
                cycle_size - orange_length,
                orange_length,
                total_length - cycle_size,
                cycle_size * (i % n_cycles) + inter_offset,
            ));

            for &lane in incoming_lanes {
                lanes[lane].control = light;
            }
        }
    }
}

impl InspectRenderDefault<LightPolicy> for LightPolicy {
    fn render(_: &[&LightPolicy], _: &'static str, _: &mut World, _: &Ui, _: &InspectArgsDefault) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut LightPolicy],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!()
        }
        let p = &mut data[0];
        let mut id = match p {
            LightPolicy::NoLights => 0,
            LightPolicy::StopSigns => 1,
            LightPolicy::Lights => 2,
            LightPolicy::Smart => 3,
        };

        let changed = imgui::ComboBox::new(&im_str!("{}", label)).build_simple_string(
            ui,
            &mut id,
            &[
                &im_str!("No lights"),
                &im_str!("Stop signs"),
                &im_str!("Lights"),
                &im_str!("Smart"),
            ],
        );

        if changed {
            match id {
                0 => **p = LightPolicy::NoLights,
                1 => **p = LightPolicy::StopSigns,
                2 => **p = LightPolicy::Lights,
                3 => **p = LightPolicy::Smart,
                _ => unreachable!(),
            }
        }

        changed
    }
}
