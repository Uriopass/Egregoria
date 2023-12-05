use crate::game_loop::Timings;
use crate::inputmap::{Bindings, InputMap};
use crate::uiworld::UiWorld;
use common::saveload::Encoder;
use egui::{Align2, Context, Widget};
use egui_extras::Column;
use engine::Fullscreen;
use engine::GfxContext;
use simulation::Simulation;
use std::time::{Duration, Instant};

const SETTINGS_SAVE_NAME: &str = "settings";

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum ShadowQuality {
    NoShadows,
    Low,
    Medium,
    High,
    TooHigh,
}

impl AsRef<str> for ShadowQuality {
    fn as_ref(&self) -> &str {
        match self {
            ShadowQuality::NoShadows => "No Shadows",
            ShadowQuality::Low => "Low",
            ShadowQuality::Medium => "Medium",
            ShadowQuality::High => "High",
            ShadowQuality::TooHigh => "Too High",
        }
    }
}

impl From<u8> for ShadowQuality {
    fn from(v: u8) -> Self {
        match v {
            0 => ShadowQuality::NoShadows,
            1 => ShadowQuality::Low,
            2 => ShadowQuality::Medium,
            3 => ShadowQuality::High,
            4 => ShadowQuality::TooHigh,
            _ => ShadowQuality::High,
        }
    }
}

impl ShadowQuality {
    pub fn size(&self) -> Option<u32> {
        match self {
            ShadowQuality::Low => Some(512),
            ShadowQuality::Medium => Some(1024),
            ShadowQuality::High => Some(2048),
            ShadowQuality::TooHigh => Some(4096),
            ShadowQuality::NoShadows => None,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub camera_border_move: bool,
    pub camera_smooth: bool,
    pub camera_smooth_tightness: f32,
    pub camera_fov: f32,

    pub fullscreen: bool,
    pub vsync: bool,
    pub ssao: bool,
    pub shadows: ShadowQuality,
    pub terrain_grid: bool,
    pub fog: bool,

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
            fullscreen: false,
            vsync: true,
            time_warp: 1,
            auto_save_every: AutoSaveEvery::FiveMinutes,
            ssao: true,
            shadows: ShadowQuality::High,
            camera_smooth_tightness: 1.0,
            camera_fov: 60.0,
            terrain_grid: true,
            fog: true,
            gui_scale: 1.0,
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

            ui.checkbox(&mut settings.fullscreen, "Fullscreen");
            ui.checkbox(&mut settings.terrain_grid, "Terrain Grid");
            ui.checkbox(&mut settings.fog, "Fog");
            ui.checkbox(&mut settings.ssao, "Ambient Occlusion (SSAO)");

            // shadow quality combobox
            let mut id = settings.shadows as u8 as usize;
            egui::ComboBox::from_label("Shadow Quality").show_index(ui, &mut id, 5, |i| {
                ShadowQuality::from(i as u8).as_ref().to_string()
            });
            settings.shadows = ShadowQuality::from(id as u8);

            ui.checkbox(&mut settings.vsync, "VSync");

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

            let mut sorted_inps = bindings.0.keys().copied().collect::<Vec<_>>();
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
    if settings.fullscreen && ctx.window.fullscreen().is_none() {
        ctx.window
            .set_fullscreen(Some(Fullscreen::Borderless(ctx.window.current_monitor())))
    }
    if !settings.fullscreen && ctx.window.fullscreen().is_some() {
        ctx.window.set_fullscreen(None);
    }

    ctx.gfx.set_vsync(settings.vsync);
    let params = ctx.gfx.render_params.value_mut();
    params.shadow_mapping_resolution = settings.shadows.size().unwrap_or(0) as i32;

    if let Some(v) = settings.shadows.size() {
        if ctx.gfx.sun_shadowmap.extent.width != v {
            ctx.gfx.sun_shadowmap = GfxContext::mk_shadowmap(&ctx.gfx.device, v);
            ctx.gfx.update_simplelit_bg();
        }
    }

    ctx.gfx.set_define_flag("FOG", settings.fog);
    ctx.gfx.set_define_flag("SSAO", settings.ssao);
    ctx.gfx
        .set_define_flag("TERRAIN_GRID", settings.terrain_grid);

    ctx.egui.pixels_per_point = settings.gui_scale;

    ctx.audio.set_settings(
        settings.master_volume_percent,
        settings.ui_volume_percent,
        settings.music_volume_percent,
        settings.effects_volume_percent,
    );
}
