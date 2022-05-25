use crate::gui::inputmap::InputMap;
use crate::uiworld::UiWorld;
use common::saveload::Encoder;
use egregoria::Egregoria;
use imgui::{Condition, Ui};
use std::time::Duration;

const SETTINGS_SAVE_NAME: &str = "settings";

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum ShadowQuality {
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
pub struct Settings {
    pub camera_border_move: bool,
    pub camera_smooth: bool,
    pub camera_smooth_tightness: f32,
    pub camera_fov: f32,

    pub fullscreen: bool,
    pub vsync: VSyncOptions,
    pub ssao: bool,
    pub shadows: ShadowQuality,
    pub realistic_sky: bool,

    pub music_volume_percent: f32,
    pub effects_volume_percent: f32,
    pub ui_volume_percent: f32,

    pub time_warp: u32,
    pub auto_save_every: AutoSaveEvery,
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
            vsync: VSyncOptions::Enabled,
            time_warp: 1,
            auto_save_every: AutoSaveEvery::Never,
            ssao: true,
            shadows: ShadowQuality::High,
            camera_smooth_tightness: 1.0,
            realistic_sky: true,
            camera_fov: 60.0,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum VSyncOptions {
    Disabled,
    Enabled,
    LowLatency,
}

impl From<VSyncOptions> for wgpu_engine::wgpu::PresentMode {
    fn from(x: VSyncOptions) -> Self {
        match x {
            VSyncOptions::Disabled => wgpu_engine::wgpu::PresentMode::Immediate,
            VSyncOptions::Enabled => wgpu_engine::wgpu::PresentMode::Fifo,
            VSyncOptions::LowLatency => wgpu_engine::wgpu::PresentMode::Mailbox,
        }
    }
}

impl AsRef<str> for VSyncOptions {
    fn as_ref(&self) -> &str {
        match self {
            VSyncOptions::Disabled => "No VSync",
            VSyncOptions::Enabled => "VSync Enabled",
            VSyncOptions::LowLatency => "Low Latency VSync",
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
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

impl AsRef<str> for AutoSaveEvery {
    fn as_ref(&self) -> &str {
        match self {
            AutoSaveEvery::Never => "Never",
            AutoSaveEvery::OneMinute => "Minute",
            AutoSaveEvery::FiveMinutes => "Five Minutes",
        }
    }
}

pub fn settings(
    window: imgui::Window<'_, &'static str>,
    ui: &Ui<'_>,
    uiworld: &mut UiWorld,
    _: &Egregoria,
) {
    let mut settings = uiworld.write::<Settings>();
    let [w, h] = ui.io().display_size;

    window
        .position([w * 0.5, h * 0.5], Condition::Appearing)
        .size([600.0, 600.0], Condition::Appearing)
        .position_pivot([0.5, 0.5])
        .movable(true)
        .collapsible(false)
        .build(ui, || {
            ui.text("Gameplay");
            let tok = imgui::ComboBox::new("Autosave")
                .preview_value(settings.auto_save_every.as_ref())
                .begin(ui);
            if let Some(tok) = tok {
                if imgui::Selectable::new("Never").build(ui) {
                    settings.auto_save_every = AutoSaveEvery::Never;
                }
                if imgui::Selectable::new("Minute").build(ui) {
                    settings.auto_save_every = AutoSaveEvery::OneMinute;
                }
                if imgui::Selectable::new("Five Minutes").build(ui) {
                    settings.auto_save_every = AutoSaveEvery::FiveMinutes;
                }
                tok.end();
            }

            ui.new_line();
            ui.text("Input");

            ui.checkbox(
                "Border screen camera movement",
                &mut settings.camera_border_move,
            );
            ui.checkbox("Camera smooth", &mut settings.camera_smooth);

            if settings.camera_smooth {
                imgui::Drag::new("Camera smoothing tightness")
                    .display_format("%.2f")
                    .speed(0.01)
                    .build(ui, &mut settings.camera_smooth_tightness);
            }
            imgui::Drag::new("Camera Field of View (FOV)")
                .display_format("%.1f")
                .range(1.0, 179.0)
                .speed(0.1)
                .build(ui, &mut settings.camera_fov);

            ui.new_line();
            ui.text("Graphics");

            ui.checkbox("Fullscreen", &mut settings.fullscreen);
            ui.checkbox("Realistic sky", &mut settings.realistic_sky);
            ui.checkbox("Ambient Occlusion (SSAO)", &mut settings.ssao);

            let tok = imgui::ComboBox::new("Shadow quality")
                .preview_value(settings.shadows.as_ref())
                .begin(ui);
            if let Some(tok) = tok {
                if imgui::Selectable::new("No Shadows").build(ui) {
                    settings.shadows = ShadowQuality::NoShadows;
                }
                if imgui::Selectable::new("Low").build(ui) {
                    settings.shadows = ShadowQuality::Low;
                }
                if imgui::Selectable::new("Medium").build(ui) {
                    settings.shadows = ShadowQuality::Medium;
                }
                if imgui::Selectable::new("High").build(ui) {
                    settings.shadows = ShadowQuality::High;
                }
                tok.end();
            }

            if let Some(tok) = imgui::ComboBox::new("VSync")
                .preview_value(settings.vsync.as_ref())
                .begin(ui)
            {
                if imgui::Selectable::new("No VSync").build(ui) {
                    settings.vsync = VSyncOptions::Disabled;
                }
                if imgui::Selectable::new("VSync Enabled").build(ui) {
                    settings.vsync = VSyncOptions::Enabled;
                }
                if imgui::Selectable::new("Low latency VSync").build(ui) {
                    settings.vsync = VSyncOptions::LowLatency;
                }
                tok.end();
            }

            ui.new_line();
            ui.text("Audio");

            imgui::Slider::new("Music volume", 0.0, 100.0)
                .display_format("%.0f")
                .build(ui, &mut settings.music_volume_percent);
            imgui::Slider::new("Effects volume", 0.0, 100.0)
                .display_format("%.0f")
                .build(ui, &mut settings.effects_volume_percent);
            imgui::Slider::new("Ui volume", 0.0, 100.0)
                .display_format("%.0f")
                .build(ui, &mut settings.ui_volume_percent);

            ui.new_line();
            ui.text("Keybinds");

            let im = uiworld.read::<InputMap>();
            ui.columns(3, &*"input_map", false);
            ui.text("Action");
            ui.next_column();
            ui.text("Input");
            ui.next_column();
            ui.next_column();

            for (act, comb) in &im.input_mapping {
                ui.text(format!("{}", act));
                ui.next_column();
                ui.text(format!("{}", comb));
                ui.next_column();
                ui.text("cannot change bindings for now");
                ui.next_column();
            }

            common::saveload::JSON::save_silent(&*settings, SETTINGS_SAVE_NAME);
        });
}
