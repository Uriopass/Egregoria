use crate::gui::lotbrush::LotBrushResource;
use crate::gui::specialbuilding::SpecialBuildingResource;
use crate::gui::windows::ImguiWindows;
use crate::gui::{InspectedEntity, RoadBuildResource, Tool, UiTex, UiTextures};
use common::GameTime;
use egregoria::engine_interaction::{KeyCode, KeyboardInfo};
use egregoria::Egregoria;
use imgui::{im_str, StyleColor, StyleVar};
use imgui::{Ui, Window};
use imgui_inspect::{InspectArgsStruct, InspectRenderStruct};
use map_model::{BuildingKind, LanePatternBuilder, LotKind};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub camera_sensibility: f32,
    pub camera_lock: bool,

    pub fullscreen: bool,

    pub music_volume_percent: f32,
    pub effects_volume_percent: f32,
    pub ui_volume_percent: f32,

    pub time_warp: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            camera_sensibility: 80.0,
            camera_lock: true,
            music_volume_percent: 100.0,
            effects_volume_percent: 100.0,
            ui_volume_percent: 100.0,
            fullscreen: true,
            time_warp: 1.0,
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

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Gui {
    pub windows: ImguiWindows,
    pub auto_save_every: AutoSaveEvery,
    #[serde(skip)]
    pub last_save: Instant,
    #[serde(skip)]
    pub last_gui_save: Instant,
    #[serde(skip)]
    pub n_cars: i32,
    pub n_pedestrians: i32,
    pub depause_warp: f32,
    pub settings: Settings,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            windows: ImguiWindows::default(),
            auto_save_every: AutoSaveEvery::Never,
            last_save: Instant::now(),
            last_gui_save: Instant::now(),
            n_cars: 100,
            n_pedestrians: 100,
            depause_warp: 1.0,
            settings: Settings::default(),
        }
    }
}

impl Gui {
    pub fn render(&mut self, ui: &Ui, goria: &mut Egregoria) {
        let tok = ui.push_style_colors(&[
            (StyleColor::WindowBg, common::config().gui_bg_col.into()),
            (StyleColor::TitleBg, common::config().gui_title_col.into()),
        ]);

        self.inspector(ui, goria);

        self.windows.render(ui, goria);

        self.menu_bar(ui, goria);

        self.toolbox(ui, goria);

        self.time_controls(ui, goria);

        self.auto_save(goria);

        tok.pop(ui);
    }

    pub fn auto_save(&mut self, goria: &mut Egregoria) {
        if let Some(every) = self.auto_save_every.into() {
            if self.last_save.elapsed() > every {
                egregoria::save_to_disk(goria);
                self.last_save = Instant::now();
            }
        }

        if self.last_gui_save.elapsed() > Duration::from_secs(1) {
            common::saveload::save_silent_json(self, "gui");
            self.last_gui_save = Instant::now();
        }
    }

    pub fn toolbox(&mut self, ui: &Ui, goria: &mut Egregoria) {
        let [w, h] = ui.io().display_size;
        let tok = ui.push_style_vars(&[
            StyleVar::WindowPadding([0.0, 0.0]),
            StyleVar::WindowBorderSize(0.0),
            StyleVar::WindowRounding(0.0),
            StyleVar::ItemSpacing([0.0, 0.0]),
        ]);

        let toolbox_w = 80.0;

        let tools = [
            (UiTex::Road, Tool::RoadbuildStraight),
            (UiTex::Curved, Tool::RoadbuildCurved),
            (UiTex::RoadEdit, Tool::RoadEditor),
            (UiTex::LotBrush, Tool::LotBrush),
            (UiTex::Buildings, Tool::SpecialBuilding),
            (UiTex::Bulldozer, Tool::Bulldozer),
        ];

        Window::new(im_str!("Toolbox"))
            .size(
                [toolbox_w, 30.0 * (tools.len() as f32) + 20.0],
                imgui::Condition::Always,
            )
            .position([w - toolbox_w, h * 0.5 - 30.0], imgui::Condition::Always)
            .scroll_bar(false)
            .title_bar(true)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .build(&ui, || {
                let cur_tool: &mut Tool = &mut goria.write::<Tool>();

                for (name, tool) in &tools {
                    let tok = ui.push_style_var(StyleVar::Alpha(
                        if std::mem::discriminant(tool) == std::mem::discriminant(cur_tool) {
                            1.0
                        } else {
                            0.6
                        },
                    ));
                    if imgui::ImageButton::new(
                        goria.read::<UiTextures>().get(*name),
                        [toolbox_w, 30.0],
                    )
                    .frame_padding(0)
                    .build(ui)
                    {
                        *cur_tool = *tool;
                    }
                    tok.pop(ui);
                }
            });
        if matches!(
            *goria.read::<Tool>(),
            Tool::RoadbuildStraight | Tool::RoadbuildCurved
        ) {
            Window::new(im_str!("Road Properties"))
                .size([150.0, 100.0], imgui::Condition::Always)
                .position(
                    [w - 150.0 - toolbox_w, h * 0.5 - 30.0],
                    imgui::Condition::Always,
                )
                .scroll_bar(false)
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(&ui, || {
                    let mut pattern = goria.write::<RoadBuildResource>().pattern_builder;

                    <LanePatternBuilder as InspectRenderStruct<LanePatternBuilder>>::render_mut(
                        &mut [&mut pattern],
                        "Road shape",
                        &ui,
                        &InspectArgsStruct {
                            header: Some(false),
                            indent_children: Some(false),
                        },
                    );

                    if pattern.n_lanes == 0 {
                        pattern.sidewalks = true;
                        pattern.parking = false;
                    }

                    goria.write::<RoadBuildResource>().pattern_builder = pattern;
                });
        }

        let brushes = [
            (im_str!("Residential"), LotKind::Residential),
            (im_str!("Commercial"), LotKind::Commercial),
        ];

        if matches!(*goria.read::<Tool>(), Tool::LotBrush) {
            Window::new(im_str!("Lot Brush"))
                .size(
                    [toolbox_w, brushes.len() as f32 * 30.0 + 20.0],
                    imgui::Condition::Always,
                )
                .position(
                    [w - toolbox_w - toolbox_w, h * 0.5 - 30.0],
                    imgui::Condition::Always,
                )
                .scroll_bar(false)
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(&ui, || {
                    let mut cur_brush = goria.write::<LotBrushResource>();

                    for (name, brush) in &brushes {
                        let tok = ui.push_style_var(StyleVar::Alpha(
                            if std::mem::discriminant(brush)
                                == std::mem::discriminant(&cur_brush.kind)
                            {
                                1.0
                            } else {
                                0.5
                            },
                        ));
                        if ui.button(name, [toolbox_w, 30.0]) {
                            cur_brush.kind = *brush;
                        }
                        tok.pop(ui);
                    }
                });
        }

        let pop_out_w = 130.0;

        if matches!(*goria.read::<Tool>(), Tool::SpecialBuilding) {
            Window::new(im_str!("Special buildings"))
                .size(
                    [
                        pop_out_w,
                        BuildingKind::SPECIAL_BUILDINGS.len() as f32 * 30.0 + 20.0,
                    ],
                    imgui::Condition::Always,
                )
                .position(
                    [w - toolbox_w - pop_out_w, h * 0.5 - 30.0],
                    imgui::Condition::Always,
                )
                .scroll_bar(false)
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(&ui, || {
                    let mut cur_build = goria.write::<SpecialBuildingResource>();

                    for (name, build) in BuildingKind::SPECIAL_BUILDINGS {
                        let tok = ui.push_style_var(StyleVar::Alpha(
                            if std::mem::discriminant(build)
                                == std::mem::discriminant(&cur_build.kind)
                            {
                                1.0
                            } else {
                                0.5
                            },
                        ));
                        if ui.button(&*im_str!("{}", name), [pop_out_w, 30.0]) {
                            cur_build.kind = *build;
                        }
                        tok.pop(ui);
                    }
                });
        }

        tok.pop(ui);
    }

    pub fn inspector(&mut self, ui: &Ui, goria: &mut Egregoria) {
        let mut inspected = *goria.read::<InspectedEntity>();
        let e = unwrap_or!(inspected.e, return);

        let mut is_open = true;
        Window::new(im_str!("Inspect"))
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([30.0, 160.0], imgui::Condition::FirstUseEver)
            .opened(&mut is_open)
            .build(&ui, || {
                inspected.dirty =
                    crate::gui::inspect::InspectRenderer { entity: e }.render(goria, ui);
            });
        if !is_open {
            inspected.e = None;
            inspected.dirty = false;
        }
        *goria.write::<InspectedEntity>() = inspected;
    }

    pub fn time_controls(&mut self, ui: &Ui, goria: &mut Egregoria) {
        let time = goria.read::<GameTime>().daytime;
        let warp = &mut self.settings.time_warp;
        let depause_warp = &mut self.depause_warp;
        if goria
            .read::<KeyboardInfo>()
            .just_pressed
            .contains(&KeyCode::Space)
        {
            if *warp == 0.0 {
                *warp = *depause_warp;
            } else {
                *depause_warp = *warp;
                *warp = 0.0;
            }
        }

        let [_, h] = ui.io().display_size;
        let tok = ui.push_style_vars(&[
            StyleVar::WindowRounding(0.0),
            StyleVar::ItemSpacing([10.0, 7.0]),
        ]);
        Window::new(im_str!("Time controls"))
            .size([165.0, 55.0], imgui::Condition::Always)
            .position([-1.0, h - 50.0], imgui::Condition::Always)
            .no_decoration()
            .collapsible(false)
            .resizable(false)
            .build(&ui, || {
                ui.text(im_str!(" Day {}", time.day));

                ui.same_line(115.0);

                ui.text(im_str!("{:02}:{:02}", time.hour, time.second));

                let red = ui.push_style_color(StyleColor::Header, [0.7, 0.2, 0.2, 0.5]);

                if imgui::Selectable::new(im_str!(" ||"))
                    .size([29.0, 15.0])
                    .selected(*warp == 0.0)
                    .build(ui)
                {
                    *depause_warp = *warp;
                    *warp = 0.0;
                }

                red.pop(ui);

                ui.same_line(0.0);

                if imgui::Selectable::new(im_str!(" 1x"))
                    .size([27.0, 15.0])
                    .selected(*warp == 1.0)
                    .build(ui)
                {
                    *warp = 1.0;
                }

                ui.same_line(0.0);

                if imgui::Selectable::new(im_str!(" 3x"))
                    .size([27.0, 15.0])
                    .selected(*warp == 3.0)
                    .build(ui)
                {
                    *warp = 3.0;
                }

                ui.same_line(0.0);

                if imgui::Selectable::new(im_str!(" Max"))
                    .size([33.0, 15.0])
                    .selected(*warp == 1000.0)
                    .build(ui)
                {
                    *warp = 1000.0;
                }
            });
        tok.pop(ui);
    }

    pub fn menu_bar(&mut self, ui: &Ui, goria: &mut Egregoria) {
        ui.main_menu_bar(|| {
            self.windows.menu(ui);

            ui.menu(im_str!("Settings"), true, || {
                ui.text("Auto save every");
                ui.same_line(0.0);
                let tok = imgui::ComboBox::new(im_str!(""))
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
                imgui::Slider::new(im_str!("Camera sensibility")).range(10.0..=200.0).display_format(im_str!("%.0f")).build(ui, &mut self.settings.camera_sensibility);
                ui.checkbox(im_str!("Camera zoom locked"), &mut self.settings.camera_lock);

                ui.checkbox(im_str!("Fullscreen"), &mut self.settings.fullscreen);
                imgui::Slider::new(im_str!("Music volume")).range(0.0..=100.0).display_format(im_str!("%.0f")).build(ui, &mut self.settings.music_volume_percent);
                imgui::Slider::new(im_str!("Effects volume")).range(0.0..=100.0).display_format(im_str!("%.0f")).build(ui, &mut self.settings.effects_volume_percent);
                imgui::Slider::new(im_str!("Ui volume")).range(0.0..=100.0).display_format(im_str!("%.0f")).build(ui, &mut self.settings.ui_volume_percent);
            });
            if ui.small_button(im_str!("Save")) {
                egregoria::save_to_disk(goria);
            }
            ui.menu(im_str!("Help"), true, || {
                ui.text(im_str!("Pan: Right click or Arrow keys"));
                ui.text(im_str!("Select: Left click"));
                ui.text(im_str!("Move: Left drag"));
                ui.text(im_str!("Deselect: Escape"));
                ui.text(im_str!("Delete (use with caution): Delete"));
                ui.text(im_str!("Use the tools on the right\nto build and modify roads"));
                ui.text(im_str!("Use the \"Map\" window to build houses\nor load prebuilt maps such as Paris\n(takes a few seconds to load)"));
            });
        });
    }
}
