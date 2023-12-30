use crate::game_loop::Timings;
use crate::inputmap::{Bindings, InputMap};
use crate::uiworld::UiWorld;
use common::saveload::Encoder;
use egui::{Align2, Context, Widget};
use egui_extras::Column;
use engine::GfxSettings;
use engine::ShadowQuality;
use simulation::Simulation;
use std::time::{Duration, Instant};

const SETTINGS_SAVE_NAME: &str = "settings";

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub camera_border_move: bool,
    pub camera_smooth: bool,
    pub camera_smooth_tightness: f32,
    pub camera_fov: f32,

    pub gfx: GfxSettings,

    pub gui_scale: f32,

    pub master_volume_percent: f32,
    pub music_volume_percent: f32,
    pub effects_volume_percent: f32,
    pub ui_volume_percent: f32,

    #[serde(skip)]
    pub time_warp: u32,
    pub auto_save_every: AutoSaveEvery,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            camera_border_move: false,
            camera_smooth: true,
            master_volume_percent: 100.0,
            music_volume_percent: 100.0,
            effects_volume_percent: 100.0,
            ui_volume_percent: 100.0,
            time_warp: 1,
            auto_save_every: AutoSaveEvery::FiveMinutes,
            camera_smooth_tightness: 1.0,
            camera_fov: 60.0,
            gui_scale: 1.0,
            gfx: GfxSettings::default(),
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum AutoSaveEvery {
    Never,
    OneMinute,
    FiveMinutes,
}

impl From<AutoSaveEvery> for Option<Duration> {
    fn from(x: AutoSaveEvery) -> Option<Duration> {
        match x {
            AutoSaveEvery::Never => None,
            AutoSaveEvery::OneMinute => Some(Duration::from_secs(60)),
            AutoSaveEvery::FiveMinutes => Some(Duration::from_secs(5 * 60)),
        }
    }
}

impl From<u8> for AutoSaveEvery {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Never,
            1 => Self::OneMinute,
            2 => Self::FiveMinutes,
            _ => Self::Never,
        }
    }
}

impl AsRef<str> for AutoSaveEvery {
    fn as_ref(&self) -> &str {
        match self {
            AutoSaveEvery::Never => "Never",
            AutoSaveEvery::OneMinute => "Minute",
            AutoSaveEvery::FiveMinutes => "Five Minutes",
        }
    }
}

/// Settings window
/// This window is used to change the settings of the game
pub fn settings(window: egui::Window<'_>, ui: &Context, uiworld: &mut UiWorld, _: &Simulation) {
    let mut settings = uiworld.write::<Settings>();
    let [_, h]: [f32; 2] = ui.available_rect().size().into();

    window
        .default_size([500.0, h * 0.8])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .vscroll(true)
        .collapsible(false)
        .show(ui, |ui| {
            let before = *settings;
            ui.label("Gameplay");

            let mut id = settings.auto_save_every as u8 as usize;
            egui::ComboBox::from_label("Autosave").show_index(ui, &mut id, 3, |i| {
                AutoSaveEvery::from(i as u8).as_ref().to_string()
            });
            settings.auto_save_every = AutoSaveEvery::from(id as u8);

            ui.label("Input");

            ui.checkbox(
                &mut settings.camera_border_move,
                "Border screen camera movement",
            );
            ui.checkbox(&mut settings.camera_smooth, "Camera smooth");

            if settings.camera_smooth {
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut settings.camera_smooth_tightness).speed(0.01));
                    ui.label("Camera smoothing tightness");
                });
            }
            ui.horizontal(|ui| {
                egui::DragValue::new(&mut settings.camera_fov)
                    .clamp_range(1.0..=179.0f32)
                    .speed(0.1)
                    .ui(ui);
                ui.label("Camera Field of View (FOV)");
            });

            let mut fps_to_show = 0.0;
            let mut ms_to_show = 0.0;
            ui.data_mut(|data| {
                let (fps, ms, instant) = data
                    .get_temp_mut_or_insert_with(ui.make_persistent_id("fps"), || {
                        (0.0, 0.0, Instant::now())
                    });
                // only update the fps every 300ms to avoid flickering
                if *fps == 0.0 || instant.elapsed() > Duration::from_millis(300) {
                    *ms = uiworld.read::<Timings>().all.avg();
                    *fps = 1.0 / *ms;
                    *instant = Instant::now();
                }

                fps_to_show = *fps;
                ms_to_show = *ms;
            });

            ui.separator();
            #[cfg(debug_assertions)]
            ui.colored_label(
                egui::Color32::BROWN,
                "shouldn't be looking at FPS in debug mode! use --release",
            );
            ui.label(format!(
                "Graphics - {fps_to_show:.1}FPS - {:.1}ms",
                1000.0 * ms_to_show
            ));

            ui.checkbox(&mut settings.gfx.fullscreen, "Fullscreen");
            ui.checkbox(&mut settings.gfx.terrain_grid, "Terrain Grid");
            ui.checkbox(&mut settings.gfx.fog, "Fog");
            ui.checkbox(&mut settings.gfx.ssao, "Ambient Occlusion (SSAO)");

            // shadow quality combobox
            let mut id = settings.gfx.shadows as u8 as usize;
            egui::ComboBox::from_label("Shadow Quality").show_index(ui, &mut id, 5, |i| {
                ShadowQuality::from(i as u8).as_ref().to_string()
            });
            settings.gfx.shadows = ShadowQuality::from(id as u8);

            ui.checkbox(&mut settings.gfx.vsync, "VSync");

            ui.separator();
            ui.label("GUI");
            ui.horizontal(|ui| {
                // we only change gui_scale at end of interaction to avoid feedback loops
                let mut gui_scale = settings.gui_scale;
                let res = ui.add(egui::Slider::new(&mut gui_scale, 0.5..=2.0));
                if res.drag_released() {
                    settings.gui_scale = gui_scale;
                }
                ui.label("GUI Scale");
            });

            ui.separator();
            ui.label("Audio");

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.master_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{x:.0}%")),
                );
                ui.label("Master volume");
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.music_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{x:.0}%")),
                );
                ui.label("Music volume");
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.effects_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{x:.0}%")),
                );
                ui.label("Effects volume");
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.ui_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{x:.0}%")),
                );
                ui.label("Ui volume");
            });

            ui.separator();
            let mut bindings = uiworld.write::<Bindings>();
            ui.horizontal(|ui| {
                ui.label("Keybinds");

                if ui.button("Reset").clicked() {
                    *bindings = Bindings::default();
                    uiworld.write::<InputMap>().build_input_tree(&mut bindings);
                }
            });

            let mut sorted_inps = bindings.0.keys().cloned().collect::<Vec<_>>();
            sorted_inps.sort();

            egui_extras::TableBuilder::new(ui)
                .column(Column::initial(150.0))
                .column(Column::initial(150.0))
                .column(Column::initial(150.0))
                .column(Column::initial(50.0))
                .header(30.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Action");
                    });
                    header.col(|ui| {
                        ui.label("Primary");
                    });
                    header.col(|ui| {
                        ui.label("Secondary");
                    });
                })
                .body(|body| {
                    body.rows(25.0, sorted_inps.len(), |i, mut ui| {
                        let action = &sorted_inps[i];
                        let comb = bindings.0.get_mut(action).unwrap();

                        ui.col(|ui| {
                            ui.label(action.to_string());
                        });
                        ui.col(|ui| {
                            let resp = if !comb.0.is_empty() {
                                ui.button(format!("{}", comb.0[0]))
                            } else {
                                ui.button("<empty>")
                            };
                            if resp.hovered() {
                                egui::show_tooltip_text(
                                    ui.ctx(),
                                    ui.make_persistent_id("notimplemented"),
                                    "Not implemented yet",
                                );
                            }
                            if resp.clicked() {}
                        });
                        ui.col(|ui| {
                            let resp = if comb.0.len() > 1 {
                                ui.button(format!("{}", comb.0[1]))
                            } else {
                                ui.button("<empty>")
                            };
                            if resp.hovered() {
                                egui::show_tooltip_text(
                                    ui.ctx(),
                                    ui.make_persistent_id("notimplemented"),
                                    "Not implemented yet",
                                );
                            }
                            if resp.clicked() {}
                        });
                        ui.col(|ui| {
                            if ui.button("↺").clicked() {
                                comb.0 = Bindings::default().0.remove(action).unwrap().0;
                            }
                        });
                    })
                });

            if *settings != before {
                common::saveload::JSONPretty::save_silent(&*settings, SETTINGS_SAVE_NAME);
            }
        });
}

pub fn manage_settings(ctx: &mut engine::Context, settings: &Settings) {
    ctx.gfx.update_settings(settings.gfx);

    ctx.egui.zoom_factor = settings.gui_scale;

    ctx.audio.set_settings(
        settings.master_volume_percent,
        settings.ui_volume_percent,
        settings.music_volume_percent,
        settings.effects_volume_percent,
    );
}
