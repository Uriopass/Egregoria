use crate::windows::ImguiWindows;
use crate::{RoadBuildResource, Tool};
use egregoria::engine_interaction::TimeInfo;

use common::inspect::InspectedEntity;
use egregoria::Egregoria;
use imgui::{im_str, StyleVar};
use imgui::{Ui, Window};
use imgui_inspect::{InspectArgsStruct, InspectRenderStruct};
use map_model::LanePatternBuilder;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

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
    #[serde(skip)]
    pub windows: ImguiWindows,
    pub auto_save_every: AutoSaveEvery,
    #[serde(skip)]
    pub last_save: Instant,
    #[serde(skip)]
    pub n_cars: i32,
    pub n_pedestrians: i32,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            windows: ImguiWindows::default(),
            auto_save_every: AutoSaveEvery::Never,
            last_save: Instant::now(),
            n_cars: 100,
            n_pedestrians: 100,
        }
    }
}

impl Gui {
    pub fn render(&mut self, ui: &Ui, goria: &mut Egregoria) {
        self.inspector(ui, goria);

        self.windows.render(ui, goria);

        self.menu_bar(ui, goria);

        self.toolbox(ui, goria);

        self.time_controls(ui, goria);

        self.auto_save(goria);
    }

    pub fn auto_save(&mut self, goria: &mut Egregoria) {
        if let Some(every) = self.auto_save_every.into() {
            let now = Instant::now();
            if now.duration_since(self.last_save) > every {
                egregoria::save_to_disk(goria);
                self.last_save = now;
            }
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

        let toolbox_w = 120.0;

        Window::new(im_str!("Toolbox"))
            .size([toolbox_w, 30.0 * 5.0 + 20.0], imgui::Condition::Always)
            .position([w - toolbox_w, h * 0.5 - 30.0], imgui::Condition::Always)
            .scroll_bar(false)
            .title_bar(true)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .build(&ui, || {
                let cur_tool: &mut Tool = &mut goria.write::<Tool>();

                let tools = [
                    (im_str!("Hand"), Tool::Hand),
                    (im_str!("Straight Road"), Tool::RoadbuildStraight),
                    (im_str!("Curved Road"), Tool::RoadbuildCurved),
                    (im_str!("Road Editor"), Tool::RoadEditor),
                    (im_str!("Bulldozer"), Tool::Bulldozer),
                ];

                for (name, tool) in &tools {
                    let tok = ui.push_style_var(StyleVar::Alpha(if tool == cur_tool {
                        1.0
                    } else {
                        0.5
                    }));
                    if ui.button(name, [toolbox_w, 30.0]) {
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

        tok.pop(ui);
    }

    pub fn inspector(&mut self, ui: &Ui, goria: &mut Egregoria) {
        let mut inspected = *goria.read::<InspectedEntity>();
        let e = match inspected.e {
            Some(x) => x,
            None => return,
        };

        let mut is_open = true;
        Window::new(im_str!("Inspect"))
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([30.0, 160.0], imgui::Condition::FirstUseEver)
            .opened(&mut is_open)
            .build(&ui, || {
                inspected.dirty = crate::inspect::InspectRenderer { entity: e }.render(goria, ui);
            });
        if !is_open {
            inspected.e = None;
            inspected.dirty = false;
        }
        *goria.write::<InspectedEntity>() = inspected;
    }

    pub fn time_controls(&mut self, ui: &Ui, goria: &mut Egregoria) {
        let mut time_info = goria.write::<TimeInfo>();
        let [w, h] = ui.io().display_size;
        Window::new(im_str!("Time controls"))
            .size([230.0, 40.0], imgui::Condition::Always)
            .position([w * 0.5 - 100.0, h - 30.0], imgui::Condition::Always)
            .no_decoration()
            .collapsible(false)
            .resizable(false)
            .build(&ui, || {
                imgui::DragFloat::new(&ui, im_str!("Time warp"), &mut time_info.time_speed)
                    .min(0.0)
                    .max(1000.0)
                    .speed(0.1)
                    .display_format(im_str!("%.1f"))
                    .build();
            });
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
            });
            if ui.small_button(im_str!("Save")) {
                egregoria::save_to_disk(goria);
            }
        });
    }
}
