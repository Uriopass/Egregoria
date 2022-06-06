use crate::gui::bulldozer::BulldozerState;
use crate::gui::inputmap::{InputAction, InputMap};
use crate::gui::lotbrush::LotBrushResource;
use crate::gui::roadeditor::RoadEditorResource;
use crate::gui::specialbuilding::{SpecialBuildKind, SpecialBuildingResource};
use crate::gui::windows::settings::Settings;
use crate::gui::windows::ImguiWindows;
use crate::gui::{InspectedEntity, RoadBuildResource, Tool, UiTex, UiTextures};
use crate::input::{KeyCode, KeyboardInfo};
use crate::uiworld::UiWorld;
use common::saveload::Encoder;
use egregoria::economy::Government;
use egregoria::souls::goods_company::GoodsCompanyRegistry;
use egregoria::utils::time::GameTime;
use egregoria::Egregoria;
use imgui::{StyleColor, StyleVar, Ui, Window};
use imgui_inspect::{
    InspectArgsDefault, InspectArgsStruct, InspectRenderDefault, InspectRenderStruct,
};
use map_model::{LanePatternBuilder, LightPolicy, LotKind, TurnPolicy};
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
    #[serde(skip)]
    pub n_pedestrians: i32,
    pub depause_warp: u32,
    #[serde(skip)]
    pub hidden: bool,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            windows: ImguiWindows::default(),
            last_save: Instant::now(),
            last_gui_save: Instant::now(),
            n_cars: 100,
            n_pedestrians: 100,
            depause_warp: 1,
            hidden: false,
        }
    }
}

impl Gui {
    pub fn render(&mut self, ui: &Ui<'_>, uiworld: &mut UiWorld, goria: &Egregoria) {
        let _tw = ui.push_style_color(StyleColor::WindowBg, common::config().gui_bg_col.into());
        let _tt = ui.push_style_color(StyleColor::TitleBg, common::config().gui_title_col.into());

        Self::inspector(ui, uiworld, goria);

        self.windows.render(ui, uiworld, goria);

        self.menu_bar(ui, uiworld, goria);

        Self::toolbox(ui, uiworld, goria);

        self.time_controls(ui, uiworld, goria);

        self.auto_save(uiworld);
    }

    pub fn auto_save(&mut self, uiworld: &mut UiWorld) {
        let every = uiworld.read::<Settings>().auto_save_every.into();
        if let Some(every) = every {
            if self.last_save.elapsed() > every {
                uiworld.please_save = true;
                uiworld.save_to_disk();
                self.last_save = Instant::now();
            }
        }

        if self.last_gui_save.elapsed() > Duration::from_secs(1) {
            common::saveload::JSON::save_silent(self, "gui");
            self.last_gui_save = Instant::now();
        }
    }

    pub fn toolbox(ui: &Ui<'_>, uiworld: &mut UiWorld, goria: &Egregoria) {
        #[derive(Copy, Clone)]
        pub enum Tab {
            Hand,
            Roadbuild,
            Roadcurved,
            Roadeditor,
            Lotbrush,
            Roadbuilding,
            Bulldozer,
            Train,
        }
        uiworld.check_present(|| Tab::Hand);

        if uiworld
            .read::<InputMap>()
            .just_act
            .contains(&InputAction::Close)
        {
            *uiworld.write::<Tool>() = Tool::Hand;
            *uiworld.write::<Tab>() = Tab::Hand;
        }

        let [w, h] = ui.io().display_size;
        let _tok1 = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        let _tok2 = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
        let _tok3 = ui.push_style_var(StyleVar::WindowRounding(0.0));
        let _tok4 = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));

        let toolbox_w = 80.0;

        let tools = [
            (UiTex::Road, Tab::Roadbuild, Tool::RoadbuildStraight),
            (UiTex::Curved, Tab::Roadcurved, Tool::RoadbuildCurved),
            (UiTex::RoadEdit, Tab::Roadeditor, Tool::RoadEditor),
            (UiTex::LotBrush, Tab::Lotbrush, Tool::LotBrush),
            (UiTex::Buildings, Tab::Roadbuilding, Tool::SpecialBuilding),
            (UiTex::Bulldozer, Tab::Bulldozer, Tool::Bulldozer),
            (UiTex::AddTrain, Tab::Train, Tool::Train),
        ];

        Window::new("Toolbox")
            .size_constraints([toolbox_w, 0.0], [toolbox_w, 1000.0])
            .position([w - toolbox_w, h * 0.5 - 30.0], imgui::Condition::Always)
            .scroll_bar(false)
            .title_bar(true)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .always_auto_resize(true)
            .build(ui, || {
                let cur_tab = *uiworld.read::<Tab>();

                for (name, tab, default_tool) in &tools {
                    let _tok = ui.push_style_var(StyleVar::Alpha(
                        if std::mem::discriminant(tab) == std::mem::discriminant(&cur_tab) {
                            1.0
                        } else {
                            0.6
                        },
                    ));
                    if imgui::ImageButton::new(
                        uiworld.read::<UiTextures>().get(*name),
                        [toolbox_w, 30.0],
                    )
                    .frame_padding(0)
                    .build(ui)
                    {
                        uiworld.insert::<Tool>(*default_tool);
                        uiworld.insert(*tab);
                    }
                }
            });

        let spacing_left = ui.push_style_var(StyleVar::WindowPadding([4.0, 4.0]));
        if matches!(*uiworld.read::<Tab>(), Tab::Roadeditor) {
            let state = &mut *uiworld.write::<RoadEditorResource>();
            if let Some(ref mut v) = state.inspect {
                let dirty = &mut state.dirty;
                Window::new("Road Properties")
                    .size([150.0, 200.0], imgui::Condition::Appearing)
                    .position(
                        [w - 150.0 - toolbox_w, h * 0.5 - 30.0],
                        imgui::Condition::Appearing,
                    )
                    .scroll_bar(false)
                    .title_bar(true)
                    .movable(false)
                    .collapsible(false)
                    .resizable(false)
                    .build(ui, || {
                        ui.text("Light policy");
                        *dirty |= <LightPolicy as InspectRenderDefault<LightPolicy>>::render_mut(
                            &mut [&mut v.light_policy],
                            "",
                            ui,
                            &InspectArgsDefault {
                                header: Some(false),
                                indent_children: Some(false),
                                ..Default::default()
                            },
                        );
                        ui.new_line();
                        ui.text("Turn policy");
                        *dirty |= <TurnPolicy as InspectRenderDefault<TurnPolicy>>::render_mut(
                            &mut [&mut v.turn_policy],
                            "Turn policy",
                            ui,
                            &InspectArgsDefault {
                                header: Some(false),
                                indent_children: Some(false),
                                ..Default::default()
                            },
                        );
                    });
            }
        }
        spacing_left.pop();

        if matches!(*uiworld.read::<Tab>(), Tab::Train) {
            let rbw = 150.0;
            Window::new("Trains")
                .size([rbw, 83.0], imgui::Condition::Appearing)
                .position(
                    [w - rbw - toolbox_w, h * 0.5 - 30.0],
                    imgui::Condition::Appearing,
                )
                .scroll_bar(false)
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    if ui.button_with_size("Add train", [rbw, 30.0]) {
                        *uiworld.write::<Tool>() = Tool::Train;
                    }

                    if ui.button_with_size("Trainstation", [rbw, 30.0]) {
                        *uiworld.write::<Tool>() = Tool::SpecialBuilding;

                        let h = LanePatternBuilder::new().rail(true).n_lanes(1).width();

                        uiworld.write::<SpecialBuildingResource>().opt = Some(SpecialBuildKind {
                            make: Box::new(move |args, commands| {
                                let d = args.obb.axis()[0].z(0.0) * 0.5;
                                let off = args.obb.axis()[1].z(0.0).normalize_to(h * 0.5 + 10.0);
                                commands.map_build_trainstation(
                                    args.mpos - d - off,
                                    args.mpos + d - off,
                                );
                            }),
                            w: h + 15.0,
                            h: 230.0,
                            asset: "assets/models/trainstation.glb".to_string(),
                            road_snap: false,
                        });
                    }
                });
        }

        if matches!(*uiworld.read::<Tab>(), Tab::Roadbuild | Tab::Roadcurved) {
            let rbw = 220.0;
            Window::new("Road Properties")
                .size([rbw, 380.0], imgui::Condition::Appearing)
                .position(
                    [w - rbw - toolbox_w, h * 0.5 - 30.0],
                    imgui::Condition::Appearing,
                )
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    let mut roadbuild = uiworld.write::<RoadBuildResource>();
                    ui.checkbox("snap to grid", &mut roadbuild.snap_to_grid);
                    if ui.button_with_size("zero ", [40.0, 23.0]) {
                        roadbuild.height_offset = 0.0;
                    }
                    ui.same_line_with_spacing(0.0, 10.0);
                    let tok = ui.push_item_width(50.0);
                    imgui::Drag::new("height off")
                        .range(0.0, 100.0)
                        .speed(1.0)
                        .display_format("%.0f")
                        .build(ui, &mut roadbuild.height_offset);
                    tok.pop(ui);
                    let pat = &mut roadbuild.pattern_builder;

                    if ui.button_with_size("Rail", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new().rail(true);
                    }

                    if ui.button_with_size("Rail one-way", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new().rail(true).one_way(true);
                    }

                    if ui.button_with_size("Street", [rbw, 30.0]) {
                        *pat = LanePatternBuilder::new();
                    }

                    if ui.button_with_size("Street one-way", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new().one_way(true);
                    }

                    if ui.button_with_size("Avenue", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new().n_lanes(2).speed_limit(13.0);
                    }

                    if ui.button_with_size("Avenue one-way", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new()
                            .n_lanes(2)
                            .one_way(true)
                            .speed_limit(13.0);
                    }

                    if ui.button_with_size("Drive", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new()
                            .parking(false)
                            .sidewalks(false)
                            .speed_limit(13.0);
                    }

                    if ui.button_with_size("Drive one-way", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new()
                            .parking(false)
                            .sidewalks(false)
                            .one_way(true)
                            .speed_limit(13.0);
                    }

                    if ui.button_with_size("Highway", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new()
                            .n_lanes(3)
                            .speed_limit(25.0)
                            .parking(false)
                            .sidewalks(false);
                    }

                    if ui.button_with_size("Highway one-way", [rbw, 30.0]) {
                        *pat = *LanePatternBuilder::new()
                            .n_lanes(3)
                            .speed_limit(25.0)
                            .parking(false)
                            .sidewalks(false)
                            .one_way(true);
                    }

                    ui.new_line();

                    if imgui::CollapsingHeader::new("custom").build(ui) {
                        <LanePatternBuilder as InspectRenderStruct<LanePatternBuilder>>::render_mut(
                            &mut [pat],
                            "Road shape",
                            ui,
                            &InspectArgsStruct {
                                header: Some(false),
                                indent_children: Some(false),
                            },
                        );

                        if pat.n_lanes == 0 {
                            pat.sidewalks = true;
                            pat.parking = false;
                        }

                        if pat.n_lanes > 10 {
                            pat.n_lanes = 10;
                        }
                    }
                });
        }

        let brushes = [("Residential", LotKind::Residential)];

        if matches!(*uiworld.read::<Tab>(), Tab::Lotbrush) {
            let lbw = 130.0;
            Window::new("Lot Brush")
                .size(
                    [lbw, 50.0 + brushes.len() as f32 * 35.0],
                    imgui::Condition::Appearing,
                )
                .position(
                    [w - toolbox_w - lbw, h * 0.5 - 30.0],
                    imgui::Condition::Appearing,
                )
                .scroll_bar(false)
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    let mut cur_brush = uiworld.write::<LotBrushResource>();

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
                        if ui.button_with_size(name, [lbw, 35.0]) {
                            cur_brush.kind = *brush;
                        }
                        tok.pop();
                    }

                    imgui::Drag::new("size")
                        .range(10.0, 300.0)
                        .display_format("%.0f")
                        .build(ui, &mut cur_brush.radius);
                });
        }

        if matches!(*uiworld.read::<Tab>(), Tab::Bulldozer) {
            let lbw = 80.0;
            Window::new("Bulldozer")
                .size_constraints([lbw, 0.0], [lbw, 1000.0])
                .position(
                    [w - toolbox_w - lbw, h * 0.5 - 30.0],
                    imgui::Condition::Appearing,
                )
                .scroll_bar(false)
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    let mut state = uiworld.write::<BulldozerState>();
                    <BulldozerState as InspectRenderDefault<BulldozerState>>::render_mut(
                        &mut [&mut *state],
                        "Bulldozer",
                        ui,
                        &InspectArgsDefault {
                            header: Some(false),
                            indent_children: Some(false),
                            ..Default::default()
                        },
                    );
                });
        }

        let building_select_w = 160.0;
        let registry = goria.read::<GoodsCompanyRegistry>();
        let gbuildings = registry.descriptions.values().peekable();

        if matches!(*uiworld.read::<Tab>(), Tab::Roadbuilding) {
            Window::new("Buildings")
                .size_constraints([building_select_w, 0.0], [building_select_w, h * 0.5])
                .position(
                    [w - toolbox_w - building_select_w, h * 0.5 - 30.0],
                    imgui::Condition::Appearing,
                )
                .title_bar(true)
                .movable(false)
                .collapsible(false)
                .resizable(false)
                .build(ui, || {
                    let mut cur_build = uiworld.write::<SpecialBuildingResource>();

                    let mut picked_descr = None;
                    for descr in gbuildings {
                        let cur_kind = cur_build.opt.as_ref().map(|x| &*x.asset).unwrap_or("");

                        let tok = ui.push_style_var(StyleVar::Alpha(
                            if descr.asset_location == cur_kind {
                                picked_descr = Some(descr);
                                1.0
                            } else {
                                0.5
                            },
                        ));
                        const SCROLLBAR_W: f32 = 10.0;
                        if ui.button_with_size(&descr.name, [building_select_w - SCROLLBAR_W, 35.0])
                            || cur_build.opt.is_none()
                        {
                            let bkind = descr.bkind;
                            let bgen = descr.bgen;
                            cur_build.opt = Some(SpecialBuildKind {
                                road_snap: true,
                                make: Box::new(move |args, commands| {
                                    if let Some(rid) = args.road_id {
                                        commands
                                            .map_build_special_building(rid, args.obb, bkind, bgen);
                                    }
                                }),
                                w: descr.size,
                                h: descr.size,
                                asset: descr.asset_location.to_string(),
                            });
                        }
                        tok.pop();
                    }

                    let bdescrpt_w = 180.0;

                    if let Some(descr) = picked_descr {
                        let _tok1 = ui.push_style_var(StyleVar::WindowPadding([5.0, 5.0]));
                        let _tok2 = ui.push_style_var(StyleVar::ItemSpacing([0.0, 3.0]));
                        Window::new("Building description")
                            .size_constraints([bdescrpt_w, 10.0], [bdescrpt_w, 10000.0])
                            .always_auto_resize(true)
                            .position(
                                [
                                    w - toolbox_w - building_select_w - bdescrpt_w,
                                    h * 0.5 - 30.0,
                                ],
                                imgui::Condition::Appearing,
                            )
                            .scroll_bar(false)
                            .title_bar(true)
                            .movable(false)
                            .collapsible(false)
                            .resizable(false)
                            .build(ui, || {
                                ui.text(format!("workers: {}", descr.n_workers));
                                ui.new_line();
                                if !descr.recipe.consumption.is_empty() {
                                    ui.text("consumption:");
                                    for (kind, n) in &descr.recipe.consumption {
                                        ui.text(format!("- {} x{}", kind, n));
                                    }
                                    ui.new_line();
                                }
                                if !descr.recipe.production.is_empty() {
                                    ui.text("production:");
                                    for (kind, n) in &descr.recipe.production {
                                        ui.text(format!("- {} x{}", kind, n));
                                    }
                                    ui.new_line();
                                }
                                ui.text(format!("time: {}s", descr.recipe.complexity));
                                ui.text(format!(
                                    "storage multiplier: {}",
                                    descr.recipe.storage_multiplier
                                ));
                            });
                    }
                });
        }
    }

    pub fn inspector(ui: &Ui<'_>, uiworld: &mut UiWorld, goria: &Egregoria) {
        let mut inspected = *uiworld.read::<InspectedEntity>();
        let e = unwrap_or!(inspected.e, return);

        let mut is_open = true;
        Window::new("Inspect")
            .size([300.0, 300.0], imgui::Condition::Appearing)
            .position([30.0, 160.0], imgui::Condition::Appearing)
            .opened(&mut is_open)
            .build(ui, || {
                let mut ins = crate::gui::inspect::InspectRenderer { entity: e };
                ins.render(uiworld, goria, ui);
                inspected.e = Some(ins.entity);
            });
        if !is_open {
            inspected.e = None;
        }
        *uiworld.write::<InspectedEntity>() = inspected;
    }

    pub fn time_controls(&mut self, ui: &Ui<'_>, uiworld: &mut UiWorld, goria: &Egregoria) {
        let time = goria.read::<GameTime>().daytime;
        let warp = &mut uiworld.write::<Settings>().time_warp;
        let depause_warp = &mut self.depause_warp;
        if uiworld
            .read::<KeyboardInfo>()
            .just_pressed
            .contains(&KeyCode::Space)
        {
            if *warp == 0 {
                *warp = *depause_warp;
            } else {
                *depause_warp = *warp;
                *warp = 0;
            }
        }

        let [_, h] = ui.io().display_size;
        let _tok1 = ui.push_style_var(StyleVar::WindowRounding(0.0));
        let _tok2 = ui.push_style_var(StyleVar::ItemSpacing([10.0, 7.0]));
        Window::new("Time controls")
            .size([165.0, 55.0], imgui::Condition::Always)
            .position([-1.0, h - 52.0], imgui::Condition::Always)
            .no_decoration()
            .collapsible(false)
            .resizable(false)
            .build(ui, || {
                ui.text(format!(" Day {}", time.day));

                ui.same_line_with_pos(115.0);

                ui.text(format!("{:02}:{:02}", time.hour, time.second));

                let red = ui.push_style_color(StyleColor::Header, [0.7, 0.2, 0.2, 0.5]);

                if imgui::Selectable::new("   ||")
                    .size([29.0, 15.0])
                    .selected(*warp == 0)
                    .build(ui)
                {
                    *depause_warp = *warp;
                    *warp = 0;
                }

                red.pop();

                ui.same_line();

                if imgui::Selectable::new("  1x")
                    .size([27.0, 15.0])
                    .selected(*warp == 1)
                    .build(ui)
                {
                    *warp = 1;
                }

                ui.same_line();

                if imgui::Selectable::new("  3x")
                    .size([27.0, 15.0])
                    .selected(*warp == 3)
                    .build(ui)
                {
                    *warp = 3;
                }

                ui.same_line();

                if imgui::Selectable::new(" Max")
                    .size([33.0, 15.0])
                    .selected(*warp == 1000)
                    .build(ui)
                {
                    *warp = 1000;
                }
            });
    }

    pub fn menu_bar(&mut self, ui: &Ui<'_>, uiworld: &mut UiWorld, goria: &Egregoria) {
        let _t = ui.push_style_var(StyleVar::ItemSpacing([3.0, 0.0]));

        ui.main_menu_bar(|| {
            self.windows.menu(ui);

            let h = ui.window_size()[1];
            if ui.button_with_size("Save", [80.0, h]) {
                uiworld.please_save = true;
                uiworld.save_to_disk();
            }

            ui.text(format!("Money: {}", goria.read::<Government>().money));
        });
    }
}
