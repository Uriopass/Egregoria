use crate::gui::lotbrush::LotBrushResource;
use crate::gui::specialbuilding::SpecialBuildingResource;
use crate::gui::windows::settings::Settings;
use crate::gui::windows::ImguiWindows;
use crate::gui::{InspectedEntity, RoadBuildResource, Tool, UiTex, UiTextures};
use crate::input::{KeyCode, KeyboardInfo};
use common::GameTime;
use egregoria::Egregoria;
use imgui::{im_str, StyleColor, StyleVar, Ui, Window};
use imgui_inspect::{InspectArgsStruct, InspectRenderStruct};
use map_model::{LanePatternBuilder, LotKind};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Gui {
    pub windows: ImguiWindows,
    #[serde(skip)]
    pub last_save: Instant,
    #[serde(skip)]
    pub last_gui_save: Instant,
    #[serde(skip)]
    pub n_cars: i32,
    pub n_pedestrians: i32,
    pub depause_warp: f32,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            windows: ImguiWindows::default(),
            last_save: Instant::now(),
            last_gui_save: Instant::now(),
            n_cars: 100,
            n_pedestrians: 100,
            depause_warp: 1.0,
        }
    }
}

impl Gui {
    pub fn render(&mut self, ui: &Ui, goria: &mut Egregoria) {
        let tok = ui.push_style_colors(&[
            (StyleColor::WindowBg, common::config().gui_bg_col.into()),
            (StyleColor::TitleBg, common::config().gui_title_col.into()),
        ]);

        Self::inspector(ui, goria);

        self.windows.render(ui, goria);

        self.menu_bar(ui, goria);

        Self::toolbox(ui, goria);

        self.time_controls(ui, goria);

        self.auto_save(goria);

        tok.pop(ui);
    }

    pub fn auto_save(&mut self, goria: &mut Egregoria) {
        let every = goria.read::<Settings>().auto_save_every.into();
        if let Some(every) = every {
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

    pub fn toolbox(ui: &Ui, goria: &mut Egregoria) {
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
            .size_constraints([toolbox_w, 0.0], [toolbox_w, 1000.0])
            .position([w - toolbox_w, h * 0.5 - 30.0], imgui::Condition::Always)
            .scroll_bar(false)
            .title_bar(true)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .always_auto_resize(true)
            .build(ui, || {
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
                .size_constraints([150.0, 0.0], [150.0, 10000.0])
                .position(
                    [w - 150.0 - toolbox_w, h * 0.5 - 30.0],
                    imgui::Condition::Always,
                )
                .scroll_bar(false)
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .always_auto_resize(true)
                .build(ui, || {
                    let mut pattern = goria.write::<RoadBuildResource>().pattern_builder;

                    <LanePatternBuilder as InspectRenderStruct<LanePatternBuilder>>::render_mut(
                        &mut [&mut pattern],
                        "Road shape",
                        ui,
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
            let lbw = 130.0;
            Window::new(im_str!("Lot Brush"))
                .size_constraints([lbw, 0.0], [lbw, 1000.0])
                .position(
                    [w - toolbox_w - lbw, h * 0.5 - 30.0],
                    imgui::Condition::Always,
                )
                .scroll_bar(false)
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .always_auto_resize(true)
                .build(ui, || {
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
                        if ui.button(name, [lbw, 35.0]) {
                            cur_brush.kind = *brush;
                        }
                        tok.pop(ui);
                    }
                });
        }

        let building_select_w = 160.0;
        let gbuildings = egregoria::souls::goods_company::GOODS_BUILDINGS;

        if matches!(*goria.read::<Tool>(), Tool::SpecialBuilding) {
            Window::new(im_str!("Buildings"))
                .size_constraints([building_select_w, 0.0], [building_select_w, h * 0.5])
                .position(
                    [w - toolbox_w - building_select_w, h * 0.5 - 30.0],
                    imgui::Condition::Always,
                )
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    let mut cur_build = goria.write::<SpecialBuildingResource>();

                    if cur_build.opt.is_none() {
                        let d = &gbuildings[0];
                        cur_build.opt =
                            Some((d.bkind, d.bgen, d.size, d.asset_location.to_string()))
                    }

                    let cur_kind = cur_build.opt.as_ref().unwrap().0;

                    let mut picked_descr = None;
                    for descr in gbuildings {
                        let tok = ui.push_style_var(StyleVar::Alpha(if descr.bkind == cur_kind {
                            picked_descr = Some(descr);
                            1.0
                        } else {
                            0.5
                        }));
                        const SCROLLBAR_W: f32 = 10.0;
                        if ui.button(
                            &im_str!("{}", descr.name),
                            [building_select_w - SCROLLBAR_W, 35.0],
                        ) {
                            cur_build.opt = Some((
                                descr.bkind,
                                descr.bgen,
                                descr.size,
                                descr.asset_location.to_string(),
                            ));
                        }
                        tok.pop(ui);
                    }

                    let bdescrpt_w = 180.0;

                    if let Some(descr) = picked_descr {
                        let tok = ui.push_style_vars(&[
                            StyleVar::WindowPadding([5.0, 5.0]),
                            StyleVar::ItemSpacing([0.0, 3.0]),
                        ]);
                        Window::new(im_str!("Building description"))
                            .size_constraints([bdescrpt_w, 10.0], [bdescrpt_w, 10000.0])
                            .always_auto_resize(true)
                            .position(
                                [
                                    w - toolbox_w - building_select_w - bdescrpt_w,
                                    h * 0.5 - 30.0,
                                ],
                                imgui::Condition::Always,
                            )
                            .scroll_bar(false)
                            .title_bar(true)
                            .movable(false)
                            .collapsible(false)
                            .resizable(false)
                            .build(ui, || {
                                ui.text(im_str!("workers: {}", descr.n_workers));
                                ui.new_line();
                                if !descr.recipe.consumption.is_empty() {
                                    ui.text("consumption:");
                                    for (kind, n) in descr.recipe.consumption {
                                        ui.text(im_str!("- {} x{}", kind, n));
                                    }
                                    ui.new_line();
                                }
                                if !descr.recipe.production.is_empty() {
                                    ui.text("production:");
                                    for (kind, n) in descr.recipe.production {
                                        ui.text(im_str!("- {} x{}", kind, n));
                                    }
                                    ui.new_line();
                                }
                                ui.text(im_str!("time: {}s", descr.recipe.complexity));
                                ui.text(im_str!(
                                    "storage multiplier: {}",
                                    descr.recipe.storage_multiplier
                                ));
                            });
                        tok.pop(ui);
                    }
                });
        }
        tok.pop(ui);
    }

    pub fn inspector(ui: &Ui, goria: &mut Egregoria) {
        let mut inspected = *goria.read::<InspectedEntity>();
        let e = unwrap_or!(inspected.e, return);

        let mut is_open = true;
        Window::new(im_str!("Inspect"))
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([30.0, 160.0], imgui::Condition::FirstUseEver)
            .opened(&mut is_open)
            .build(ui, || {
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
        let warp = &mut goria.write::<Settings>().time_warp;
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
            .position([-1.0, h - 52.0], imgui::Condition::Always)
            .no_decoration()
            .collapsible(false)
            .resizable(false)
            .build(ui, || {
                ui.text(im_str!(" Day {}", time.day));

                ui.same_line(115.0);

                ui.text(im_str!("{:02}:{:02}", time.hour, time.second));

                let red = ui.push_style_color(StyleColor::Header, [0.7, 0.2, 0.2, 0.5]);

                if imgui::Selectable::new(im_str!("   ||"))
                    .size([29.0, 15.0])
                    .selected(*warp == 0.0)
                    .build(ui)
                {
                    *depause_warp = *warp;
                    *warp = 0.0;
                }

                red.pop(ui);

                ui.same_line(0.0);

                if imgui::Selectable::new(im_str!("  1x"))
                    .size([27.0, 15.0])
                    .selected(*warp == 1.0)
                    .build(ui)
                {
                    *warp = 1.0;
                }

                ui.same_line(0.0);

                if imgui::Selectable::new(im_str!("  3x"))
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
        let t = ui.push_style_vars(&[StyleVar::ItemSpacing([3.0, 0.0])]);

        ui.main_menu_bar(|| {
            self.windows.menu(ui);

            let h = ui.window_size()[1];
            if ui.button(im_str!("Save"), [80.0, h]) {
                egregoria::save_to_disk(goria);
            }

            ui.menu(im_str!("Help"), true, || {
                ui.text(im_str!("Pan: Right click or Arrow keys"));
                ui.text(im_str!("Select: Left click"));
                ui.text(im_str!("Deselect: Escape"));
                ui.text(im_str!("Delete (use with caution): Delete"));
                ui.text(im_str!("Use the tools on the right\nto build and modify roads"));
                ui.text(im_str!("Use the \"Map\" window to build houses\nor load prebuilt maps such as Paris\n(takes a few seconds to load)"));
            });
        });
        t.pop(ui);
    }
}
