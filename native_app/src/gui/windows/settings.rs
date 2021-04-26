use crate::uiworld::UiWorld;
use common::saveload::Encoder;
use egregoria::Egregoria;
use imgui::{im_str, Condition, Ui};
use std::time::Duration;

const SETTINGS_SAVE_NAME: &str = "settings";

register_resource!(Settings, SETTINGS_SAVE_NAME);

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub camera_border_move: bool,
    pub camera_smooth: bool,

    pub fullscreen: bool,
    pub vsync: VSyncOptions,
    pub ssao: bool,

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
            fullscreen: true,
            vsync: VSyncOptions::Vsync,
            time_warp: 1,
            auto_save_every: AutoSaveEvery::Never,
            ssao: true,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum VSyncOptions {
    NoVsync,
    Vsync,
    LowLatencyVsync,
}

impl From<VSyncOptions> for wgpu_engine::wgpu::PresentMode {
    fn from(x: VSyncOptions) -> Self {
        match x {
            VSyncOptions::NoVsync => wgpu_engine::wgpu::PresentMode::Immediate,
            VSyncOptions::Vsync => wgpu_engine::wgpu::PresentMode::Fifo,
            VSyncOptions::LowLatencyVsync => wgpu_engine::wgpu::PresentMode::Mailbox,
        }
    }
}

impl AsRef<str> for VSyncOptions {
    fn as_ref(&self) -> &str {
        match self {
            VSyncOptions::NoVsync => "No VSync",
            VSyncOptions::Vsync => "VSync Enabled",
            VSyncOptions::LowLatencyVsync => "Low Latency VSync",
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

pub fn settings(window: imgui::Window, ui: &Ui, uiworld: &mut UiWorld, _: &Egregoria) {
    let mut settings = uiworld.write::<Settings>();
    let [w, h] = ui.io().display_size;

    window
        .position([w * 0.5, h * 0.5], Condition::Always)
        .position_pivot([0.5, 0.5])
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .size_constraints([400.0, h * 0.6], [(w * 0.6).max(400.0), h * 0.8])
        .build(ui, || {
            ui.text("Gameplay");
            let tok = imgui::ComboBox::new(im_str!("Autosave"))
                .preview_value(&im_str!("{}", settings.auto_save_every.as_ref()))
                .begin(ui);
            if let Some(tok) = tok {
                if imgui::Selectable::new(im_str!("Never")).build(ui) {
                    settings.auto_save_every = AutoSaveEvery::Never;
                }
                if imgui::Selectable::new(im_str!("Minute")).build(ui) {
                    settings.auto_save_every = AutoSaveEvery::OneMinute;
                }
                if imgui::Selectable::new(im_str!("Five Minutes")).build(ui) {
                    settings.auto_save_every = AutoSaveEvery::FiveMinutes;
                }
                tok.end(ui);
            }

            ui.new_line();
            ui.text("Input");

            ui.checkbox(
                im_str!("Border screen camera movement"),
                &mut settings.camera_border_move,
            );
            ui.checkbox(im_str!("Camera smooth"), &mut settings.camera_smooth);

            ui.new_line();
            ui.text("Graphics");

            ui.checkbox(im_str!("Fullscreen"), &mut settings.fullscreen);
            ui.checkbox(im_str!("Ambient Occlusion (SSAO)"), &mut settings.ssao);

            if let Some(tok) = imgui::ComboBox::new(im_str!("VSync"))
                .preview_value(&im_str!("{}", settings.vsync.as_ref()))
                .begin(ui)
            {
                if imgui::Selectable::new(im_str!("No VSync")).build(ui) {
                    settings.vsync = VSyncOptions::NoVsync;
                }
                if imgui::Selectable::new(im_str!("VSync Enabled")).build(ui) {
                    settings.vsync = VSyncOptions::Vsync;
                }
                if imgui::Selectable::new(im_str!("Low latency VSync")).build(ui) {
                    settings.vsync = VSyncOptions::LowLatencyVsync;
                }
                tok.end(ui);
            }

            ui.new_line();
            ui.text("Audio");

            imgui::Slider::new(im_str!("Music volume"))
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f"))
                .build(ui, &mut settings.music_volume_percent);
            imgui::Slider::new(im_str!("Effects volume"))
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f"))
                .build(ui, &mut settings.effects_volume_percent);
            imgui::Slider::new(im_str!("Ui volume"))
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f"))
                .build(ui, &mut settings.ui_volume_percent);

            common::saveload::JSON::save_silent(&*settings, SETTINGS_SAVE_NAME);
        });
}
