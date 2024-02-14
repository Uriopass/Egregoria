#![allow(unused)]
use crate::uiworld::{SaveLoadState, UiWorld};
use egui::{Color32, DroppedFile, Widget};
use goryak::{
    button_primary, error, minrow, on_primary, on_secondary_container, primary, textc, ProgressBar,
    Window,
};
use simulation::utils::scheduler::SeqSchedule;
use simulation::Simulation;
use std::path::PathBuf;
use yakui::widgets::Pad;
use yakui::{Color, Vec2};

pub struct LoadState {
    curpath: Option<PathBuf>,
    load_fail: String,
    has_save: bool,
}

impl Default for LoadState {
    fn default() -> Self {
        Self {
            curpath: None,
            load_fail: String::new(),
            has_save: std::fs::metadata("world/world_replay.json").is_ok(),
        }
    }
}

/// Load window
/// Allows to load a replay from disk and play it
pub fn load(uiw: &UiWorld, _: &Simulation, opened: &mut bool) {
    Window {
        title: "Load".into(),
        pad: Pad::all(10.0),
        radius: 10.0,
        opened,
        child_spacing: 10.0,
    }
    .show(|| {
        let mut state = uiw.write::<LoadState>();

        if button_primary("New Game").show().clicked {
            uiw.write::<SaveLoadState>().please_load_sim = Some(Simulation::new(true));
        }

        if state.has_save {
            if button_primary("Load world/world_replay.json")
                .show()
                .clicked
            {
                let replay = Simulation::load_replay_from_disk("world");

                if let Some(replay) = replay {
                    let (mut sim, mut loader) = Simulation::from_replay(replay);
                    let mut s = SeqSchedule::default();
                    loader.advance_tick(&mut sim, &mut s); // advance by one tick to get the initial state (like map size info)

                    uiw.write::<SaveLoadState>().please_load = Some(loader);
                    uiw.write::<SaveLoadState>().please_load_sim = Some(sim);
                } else {
                    state.load_fail = "Failed to load replay".to_string();
                }
            }
        } else {
            textc(
                on_secondary_container(),
                "No replay found in world/world_replay.json",
            );
        }

        if let Some(ref mut loading) = uiw.write::<SaveLoadState>().please_load {
            let ticks_done = loading.pastt.0;
            let ticks_total = loading.replay.last_tick_recorded.0;
            ProgressBar {
                value: ticks_done as f32 / ticks_total as f32,
                size: Vec2::new(400.0, 25.0),
                color: primary().adjust(0.7),
            }
            .show_children(|| {
                textc(
                    on_secondary_container(),
                    format!("Loading replay: {ticks_done}/{ticks_total}"),
                );
            });

            minrow(5.0, || {
                if button_primary("||").show().clicked {
                    loading.speed = 0;
                }

                if button_primary(">").show().clicked {
                    loading.speed = 1;
                }
                if button_primary(">>>").show().clicked {
                    loading.speed = 100;
                }
                if button_primary("max").show().clicked {
                    loading.speed = 10000;
                }

                if button_primary("1").show().clicked {
                    loading.speed = 0;
                    loading.advance_n_ticks = 1;
                }
                if button_primary("10").show().clicked {
                    loading.speed = 0;
                    loading.advance_n_ticks = 10;
                }
                if button_primary("100").show().clicked {
                    loading.speed = 0;
                    loading.advance_n_ticks = 100;
                }
                if button_primary("1000").show().clicked {
                    loading.speed = 0;
                    loading.advance_n_ticks = 1000;
                }
            });
        }

        if !state.load_fail.is_empty() {
            textc(error(), state.load_fail.clone());
        }
    });
}
