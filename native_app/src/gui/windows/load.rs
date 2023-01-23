#![allow(unused)]
use crate::uiworld::{SaveLoadState, UiWorld};
use egregoria::Egregoria;
use egui::{Color32, DroppedFile};
use std::path::PathBuf;

#[derive(Default)]
pub struct LoadState {
    curpath: Option<PathBuf>,
    load_fail: String,
}

pub(crate) fn load(window: egui::Window<'_>, ui: &egui::Context, uiw: &mut UiWorld, _: &Egregoria) {
    window.show(ui, |ui| {
        let mut lstate = uiw.write::<LoadState>();
        /*
        ui.label("Drop a file anywhere");

        let inp = ui.input();
        let dropped_files: &Vec<DroppedFile> = &inp.raw.dropped_files;
        for file in dropped_files {
            let Some(ref path) = file.path else { continue };
            lstate.curpath = Some(path.clone());
        }
        drop(inp);

        if let Some(ref path) = lstate.curpath {
            ui.label(format!("path: {path:?}"));
        }

        if ui.button("Load").clicked() {
            let replay = Egregoria::load_replay_from_disk("world");

            if replay.is_none() {
                lstate.load_fail = "Failed to load replay".to_string();
            } else {
                uiw.write::<SaveLoadState>().please_load = replay;
            }
        }*/

        if std::fs::metadata("world/world_replay.json").is_ok() {
            if ui.button("Load world/world_replay.json").clicked() {
                let replay = Egregoria::load_replay_from_disk("world");

                if replay.is_none() {
                    lstate.load_fail = "Failed to load replay".to_string();
                } else {
                    uiw.write::<SaveLoadState>().please_load = replay;
                }
            }
        } else {
            ui.label("No replay found in world/world_replay.json");
        }

        if !lstate.load_fail.is_empty() {
            ui.colored_label(Color32::RED, &lstate.load_fail);
        }
    });
}
