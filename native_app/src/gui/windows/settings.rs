use crate::inputmap::InputMap;
use crate::uiworld::UiWorld;
use common::saveload::Encoder;
use egregoria::Egregoria;
use egui::{Align2, Context, Widget};
use std::time::Duration;

const SETTINGS_SAVE_NAME: &str = "settings";

#[derive(Serialize, Deserialize, Copy, Clone)]
pub(crate) enum ShadowQuality {
    NoShadows,
    Low,
    Medium,
    High,
}

impl AsRef<str> for ShadowQuality {
    fn as_ref(&self) -> &str {
        match self {
            ShadowQuality::NoShadows => "No Shadows",
            ShadowQuality::Low => "Low",
            ShadowQuality::Medium => "Medium",
            ShadowQuality::High => "High",
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
            _ => ShadowQuality::High,
        }
    }
}

impl ShadowQuality {
    pub(crate) fn size(&self) -> Option<u32> {
        match self {
            ShadowQuality::Low => Some(512),
            ShadowQuality::Medium => Some(1024),
            ShadowQuality::High => Some(2048),
            ShadowQuality::NoShadows => None,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub(crate) camera_border_move: bool,
    pub(crate) camera_smooth: bool,
    pub(crate) camera_smooth_tightness: f32,
    pub(crate) camera_fov: f32,

    pub(crate) fullscreen: bool,
    pub(crate) vsync: bool,
    pub(crate) ssao: bool,
    pub(crate) shadows: ShadowQuality,
    pub(crate) realistic_sky: bool,
    pub(crate) terrain_grid: bool,

    pub(crate) music_volume_percent: f32,
    pub(crate) effects_volume_percent: f32,
    pub(crate) ui_volume_percent: f32,

    pub(crate) time_warp: u32,
    pub(crate) auto_save_every: AutoSaveEvery,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            camera_border_move: true,
            camera_smooth: true,
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
            realistic_sky: true,
            camera_fov: 60.0,
            terrain_grid: true,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub(crate) enum AutoSaveEvery {
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

pub(crate) fn settings(
    window: egui::Window<'_>,
    ui: &Context,
    uiworld: &mut UiWorld,
    _: &Egregoria,
) {
    let mut settings = uiworld.write::<Settings>();
    let [w, h]: [f32; 2] = ui.available_rect().size().into();

    window
        .default_pos([w * 0.5, h * 0.5])
        .default_size([600.0, 600.0])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .show(ui, |ui| {
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

            let fps = 60.0;

            ui.separator();
            ui.label(format!("Graphics - {:.1}FPS", fps));

            ui.checkbox(&mut settings.fullscreen, "Fullscreen");
            ui.checkbox(&mut settings.realistic_sky, "Realistic sky");
            ui.checkbox(&mut settings.terrain_grid, "Terrain Grid");
            ui.checkbox(&mut settings.ssao, "Ambient Occlusion (SSAO)");

            // shadow quality combobox
            let mut id = settings.shadows as u8 as usize;
            egui::ComboBox::from_label("Shadow Quality").show_index(ui, &mut id, 3, |i| {
                ShadowQuality::from(i as u8).as_ref().to_string()
            });
            settings.shadows = ShadowQuality::from(id as u8);

            ui.checkbox(&mut settings.vsync, "VSync");

            ui.separator();
            ui.label("Audio");

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.music_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{:.0}%", x)),
                );
                ui.label("Music volume");
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.effects_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{:.0}%", x)),
                );
                ui.label("Effects volume");
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.ui_volume_percent, 0.0..=100.0)
                        .custom_formatter(|x, _| format!("{:.0}%", x)),
                );
                ui.label("Ui volume");
            });

            ui.separator();
            ui.label("Keybinds");

            let im = uiworld.read::<InputMap>();

            ui.columns(3, |ui| {
                ui[0].label("Action");
                ui[1].label("Input");
                ui[2].label("...");

                let mut sorted_inps = im.input_mapping.keys().collect::<Vec<_>>();
                sorted_inps.sort();
                for (act, comb) in sorted_inps
                    .into_iter()
                    .map(|x| (x, im.input_mapping.get(x).unwrap()))
                {
                    ui[0].label(format!("{}", act));
                    ui[1].label(format!("{}", comb));
                    ui[2].label("cannot change bindings for now");
                }
            });

            common::saveload::JSON::save_silent(&*settings, SETTINGS_SAVE_NAME);
        });
}
