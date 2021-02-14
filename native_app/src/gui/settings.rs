use imgui::im_str;
use imgui::Ui;
use std::time::Duration;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub camera_sensibility: f32,
    pub camera_lock: bool,
    pub camera_border_move: bool,

    pub fullscreen: bool,
    pub vsync: VSyncOptions,

    pub music_volume_percent: f32,
    pub effects_volume_percent: f32,
    pub ui_volume_percent: f32,

    pub time_warp: f32,
    pub auto_save_every: AutoSaveEvery,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            camera_sensibility: 80.0,
            camera_lock: true,
            camera_border_move: true,
            music_volume_percent: 100.0,
            effects_volume_percent: 100.0,
            ui_volume_percent: 100.0,
            fullscreen: true,
            vsync: VSyncOptions::Vsync,
            time_warp: 1.0,
            auto_save_every: AutoSaveEvery::Never,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum VSyncOptions {
    NoVsync,
    Vsync,
    LowLatencyVsync,
}

impl Into<wgpu_engine::wgpu::PresentMode> for VSyncOptions {
    fn into(self) -> wgpu_engine::wgpu::PresentMode {
        match self {
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

impl Into<Option<Duration>> for AutoSaveEvery {
    fn into(self) -> Option<Duration> {
        match self {
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

impl Settings {
    pub fn menu<'a>(&'a mut self, ui: &'a Ui) -> impl FnOnce() + 'a {
        move || {
            ui.text("Gameplay");
            let tok = imgui::ComboBox::new(im_str!("Autosave"))
                .preview_value(&im_str!("{}", self.auto_save_every.as_ref()))
                .begin(ui);
            if let Some(tok) = tok {
                if imgui::Selectable::new(im_str!("Never")).build(ui) {
                    self.auto_save_every = AutoSaveEvery::Never;
                }
                if imgui::Selectable::new(im_str!("Minute")).build(ui) {
                    self.auto_save_every = AutoSaveEvery::OneMinute;
                }
                if imgui::Selectable::new(im_str!("Five Minutes")).build(ui) {
                    self.auto_save_every = AutoSaveEvery::FiveMinutes;
                }
                tok.end(ui);
            }

            ui.new_line();
            ui.text("Input");

            imgui::Slider::new(im_str!("Camera sensibility"))
                .range(10.0..=200.0)
                .display_format(im_str!("%.0f"))
                .build(ui, &mut self.camera_sensibility);
            ui.checkbox(im_str!("Camera zoom locked"), &mut self.camera_lock);
            ui.checkbox(
                im_str!("Border screen camera movement"),
                &mut self.camera_border_move,
            );

            ui.new_line();
            ui.text("Graphics");

            ui.checkbox(im_str!("Fullscreen"), &mut self.fullscreen);

            if let Some(tok) = imgui::ComboBox::new(im_str!("VSync"))
                .preview_value(&im_str!("{}", self.vsync.as_ref()))
                .begin(ui)
            {
                if imgui::Selectable::new(im_str!("No VSync")).build(ui) {
                    self.vsync = VSyncOptions::NoVsync;
                }
                if imgui::Selectable::new(im_str!("VSync Enabled")).build(ui) {
                    self.vsync = VSyncOptions::Vsync;
                }
                if imgui::Selectable::new(im_str!("Low latency VSync")).build(ui) {
                    self.vsync = VSyncOptions::LowLatencyVsync;
                }
                tok.end(ui);
            }

            ui.new_line();
            ui.text("Audio");

            imgui::Slider::new(im_str!("Music volume"))
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f"))
                .build(ui, &mut self.music_volume_percent);
            imgui::Slider::new(im_str!("Effects volume"))
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f"))
                .build(ui, &mut self.effects_volume_percent);
            imgui::Slider::new(im_str!("Ui volume"))
                .range(0.0..=100.0)
                .display_format(im_str!("%.0f"))
                .build(ui, &mut self.ui_volume_percent);
        }
    }
}
