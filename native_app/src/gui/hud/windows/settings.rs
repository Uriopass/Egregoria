use std::time::{Duration, Instant};

use yakui::widgets::{CountGrid, List, Pad};
use yakui::{
    constrained, divider, Constraints, CrossAxisAlignment, MainAxisAlignItems, MainAxisSize, Vec2,
};

use common::saveload::Encoder;
use engine::GfxSettings;
use engine::ShadowQuality;
use goryak::{
    button_primary, checkbox_value, combo_box, dragvalue, icon_button, minrow,
    on_secondary_container, outline, padx, padxy, textc, VertScrollSize, Window,
};
use serde::{Deserialize, Serialize};
use simulation::Simulation;

use crate::game_loop::Timings;
use crate::gui::keybinds::{KeybindState, KeybindStateInner};
use crate::inputmap::{Bindings, InputMap};
use crate::uiworld::UiWorld;

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
    Never = 0,
    OneMinute = 1,
    FiveMinutes = 2,
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

pub struct SettingsState {
    fps: f32,
    ms: f32,
    instant: Instant,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            fps: 0.0,
            ms: 0.0,
            instant: Instant::now(),
        }
    }
}

/// Settings window
/// This window is used to change the settings of the game
pub fn settings(uiw: &UiWorld, _: &Simulation, opened: &mut bool) {
    Window {
        title: "Settings".into(),
        pad: Pad::all(10.0),
        radius: 10.0,
        opened,
        child_spacing: 0.0,
    }
    .show(|| {
        profiling::scope!("gui::window::settings");

        VertScrollSize::Percent(0.8).show(|| {
            let mut l = List::column();
            l.item_spacing = 5.0;
            l.main_axis_size = MainAxisSize::Min;
            l.show(|| {
                let mut settings = uiw.write::<Settings>();
                let mut state = uiw.write::<SettingsState>();
                let before = *settings;

                textc(on_secondary_container(), "Gameplay");
                minrow(5.0, || {
                    textc(on_secondary_container(), "Auto save every");
                    let mut id = settings.auto_save_every as u8 as usize;
                    if combo_box(
                        &mut id,
                        &[
                            AutoSaveEvery::Never.as_ref(),
                            AutoSaveEvery::OneMinute.as_ref(),
                            AutoSaveEvery::FiveMinutes.as_ref(),
                        ],
                        200.0,
                    ) {
                        settings.auto_save_every = AutoSaveEvery::from(id as u8);
                    }
                });

                divider(outline(), 10.0, 1.0);
                textc(on_secondary_container(), "Input");
                checkbox_value(
                    &mut settings.camera_border_move,
                    on_secondary_container(),
                    "Border screen camera movement",
                );
                checkbox_value(
                    &mut settings.camera_smooth,
                    on_secondary_container(),
                    "Camera smooth",
                );

                if settings.camera_smooth {
                    minrow(5.0, || {
                        dragvalue()
                            .min(0.1)
                            .max(2.0)
                            .step(0.1)
                            .show(&mut settings.camera_smooth_tightness);
                        textc(on_secondary_container(), "Camera smoothing tightness");
                    });
                }

                minrow(5.0, || {
                    dragvalue()
                        .min(2.0)
                        .max(179.0)
                        .step(1.0)
                        .show(&mut settings.camera_fov);
                    textc(on_secondary_container(), "Camera Field of View (FOV)");
                });

                // only update the fps every 300ms to avoid flickering
                if state.fps == 0.0 || state.instant.elapsed() > Duration::from_millis(300) {
                    state.ms = uiw.read::<Timings>().all.avg();
                    state.fps = 1.0 / state.ms;
                    state.instant = Instant::now();
                }

                divider(outline(), 10.0, 1.0);
                #[cfg(debug_assertions)]
                textc(
                    on_secondary_container(),
                    "shouldn't be looking at FPS in debug mode! use --release",
                );
                textc(
                    on_secondary_container(),
                    format!(
                        "Graphics - {:.1}FPS - {:.1}ms",
                        state.fps,
                        1000.0 * state.ms
                    ),
                );
                checkbox_value(
                    &mut settings.gfx.fullscreen,
                    on_secondary_container(),
                    "Fullscreen",
                );
                checkbox_value(
                    &mut settings.gfx.terrain_grid,
                    on_secondary_container(),
                    "Terrain Grid",
                );
                checkbox_value(&mut settings.gfx.fog, on_secondary_container(), "Fog");
                checkbox_value(
                    &mut settings.gfx.ssao,
                    on_secondary_container(),
                    "Ambient Occlusion (SSAO)",
                );
                checkbox_value(
                    &mut settings.gfx.msaa,
                    on_secondary_container(),
                    "MSAA 4x Anti-aliasing",
                );
                checkbox_value(&mut settings.gfx.vsync, on_secondary_container(), "VSync");
                checkbox_value(
                    &mut settings.gfx.parallel_render,
                    on_secondary_container(),
                    "Threaded rendering",
                );

                minrow(5.0, || {
                    let mut id = settings.gfx.shadows as u8 as usize;
                    if combo_box(
                        &mut id,
                        &[
                            ShadowQuality::NoShadows.as_ref(),
                            ShadowQuality::Low.as_ref(),
                            ShadowQuality::Medium.as_ref(),
                            ShadowQuality::High.as_ref(),
                            ShadowQuality::Ultra.as_ref(),
                        ],
                        200.0,
                    ) {
                        settings.gfx.shadows = ShadowQuality::from(id as u8);
                    }
                    textc(on_secondary_container(), "Shadow Quality");
                });

                divider(outline(), 10.0, 1.0);
                textc(on_secondary_container(), "GUI");
                minrow(5.0, || {
                    dragvalue().min(0.5).max(2.0).show(&mut settings.gui_scale);
                    textc(on_secondary_container(), "GUI Scale");
                });

                divider(outline(), 10.0, 1.0);
                textc(on_secondary_container(), "Audio");
                minrow(5.0, || {
                    dragvalue()
                        .min(0.0)
                        .max(100.0)
                        .step(1.0)
                        .show(&mut settings.master_volume_percent);
                    textc(on_secondary_container(), "Master volume");
                });

                minrow(5.0, || {
                    dragvalue()
                        .min(0.0)
                        .max(100.0)
                        .step(1.0)
                        .show(&mut settings.music_volume_percent);
                    textc(on_secondary_container(), "Music volume");
                });

                minrow(5.0, || {
                    dragvalue()
                        .min(0.0)
                        .max(100.0)
                        .step(1.0)
                        .show(&mut settings.effects_volume_percent);
                    textc(on_secondary_container(), "Effects volume");
                });

                minrow(5.0, || {
                    dragvalue()
                        .min(0.0)
                        .max(100.0)
                        .step(1.0)
                        .show(&mut settings.ui_volume_percent);
                    textc(on_secondary_container(), "Ui volume");
                });

                divider(outline(), 10.0, 1.0);
                textc(on_secondary_container(), "Keybinds");
                let mut bindings = uiw.write::<Bindings>();
                if button_primary("Reset").show().clicked {
                    *bindings = Bindings::default();
                    uiw.write::<InputMap>().build_input_tree(&mut bindings);
                }

                let mut sorted_inps = bindings.0.keys().cloned().collect::<Vec<_>>();
                sorted_inps.sort();

                constrained(
                    Constraints::loose(Vec2::new(f32::INFINITY, 100000.0)),
                    || {
                        CountGrid::col(4)
                            .main_axis_size(MainAxisSize::Min)
                            .cross_axis_aligment(CrossAxisAlignment::Start)
                            .main_axis_align_items(MainAxisAlignItems::Center)
                            .show(|| {
                                for action in &sorted_inps {
                                    let comb = bindings.0.get_mut(action).unwrap();
                                    padx(2.0, || {
                                        textc(on_secondary_container(), action.to_string());
                                    });
                                    let print_comb = |index: usize| {
                                        padx(2.0, || {
                                            minrow(0.0, || {
                                                let resp = if comb.0.len() > index {
                                                    button_primary(format!("{}", comb.0[index]))
                                                        .show()
                                                } else {
                                                    button_primary("<empty>").show()
                                                };
                                                if resp.clicked {
                                                    let mut state = uiw.write::<KeybindState>();
                                                    state.enabled = Some(KeybindStateInner {
                                                        to_bind_to: action.clone(),
                                                        cur: Default::default(),
                                                        bind_index: index,
                                                    });
                                                }
                                            });
                                        });
                                    };
                                    print_comb(0);
                                    print_comb(1);
                                    padxy(8.0, 2.0, || {
                                        minrow(0.0, || {
                                            if icon_button(button_primary("arrows-rotate"))
                                                .show()
                                                .clicked
                                            {
                                                comb.0 =
                                                    Bindings::default().0.remove(action).unwrap().0;
                                            }
                                        });
                                    });
                                }
                            });
                    },
                );

                if *settings != before {
                    common::saveload::JSONPretty::save_silent(&*settings, SETTINGS_SAVE_NAME);
                }
            });
        });
    })
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
