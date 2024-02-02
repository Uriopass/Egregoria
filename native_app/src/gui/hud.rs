use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use egui::load::SizedTexture;
use egui::{Align2, Color32, Context, Frame, Id, Response, RichText, Style, Ui, Widget, Window};
use serde::{Deserialize, Serialize};

use common::saveload::Encoder;
use geom::{Polygon, Vec2};
use prototypes::{
    prototypes_iter, BuildingGen, FreightStationPrototype, GoodsCompanyPrototype, ItemID, Money,
};
use simulation::economy::Government;
use simulation::map::{BuildingKind, LanePatternBuilder, MapProject, Zone};
use simulation::world_command::WorldCommand;
use simulation::Simulation;

use crate::gui::chat::chat;
use crate::gui::inspect::inspector;
use crate::gui::windows::settings::Settings;
use crate::gui::windows::GUIWindows;
use crate::gui::UiTextures;
use crate::inputmap::{InputAction, InputMap};
use crate::newgui::specialbuilding::{SpecialBuildKind, SpecialBuildingResource};
use crate::newgui::{ErrorTooltip, PotentialCommands, Tool};
use crate::uiworld::{SaveLoadState, UiWorld};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Gui {
    pub windows: GUIWindows,
    #[serde(skip)]
    pub last_save: Instant,
    #[serde(skip)]
    pub last_gui_save: Instant,
    #[serde(skip)]
    pub n_cars: i32,
    #[serde(skip)]
    pub n_pedestrians: i32,
    #[serde(skip)]
    pub depause_warp: u32,
    #[serde(skip)]
    pub hidden: bool,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            windows: GUIWindows::default(),
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
    pub fn set_style(ui: &Context) {
        let mut style: Style = (*ui.style()).clone();
        style.visuals.window_shadow.extrusion = 2.0;
        ui.set_style(style);
    }

    /// Root GUI entrypoint
    pub fn render(&mut self, ui: &Context, uiworld: &mut UiWorld, sim: &Simulation) {
        profiling::scope!("hud::render");
        self.auto_save(uiworld);

        if self.hidden {
            return;
        }

        self.menu_bar(ui, uiworld, sim);

        inspector(ui, uiworld, sim);

        chat(ui, uiworld, sim);

        self.windows.render(ui, uiworld, sim);

        Self::toolbox(ui, uiworld, sim);

        self.tooltip(ui, uiworld, sim);
    }

    pub fn tooltip(&mut self, ui: &Context, uiworld: &mut UiWorld, sim: &Simulation) {
        profiling::scope!("gui::tooltip");
        let tooltip = std::mem::take(&mut *uiworld.write::<ErrorTooltip>());
        if let Some(msg) = tooltip.msg {
            if !(tooltip.isworld && ui.is_pointer_over_area()) {
                let s = ui.available_rect().size();
                egui::show_tooltip_at(
                    ui,
                    Id::new("tooltip_error"),
                    Some(egui::Pos2::new(s.x, s.y)),
                    |ui| ui.label(RichText::new(msg).color(Color32::from_rgb(255, 100, 100))),
                );
            }
        }

        if ui.is_pointer_over_area() {
            return;
        }
        let pot = &mut uiworld.write::<PotentialCommands>().0;
        let cost: Money = pot
            .drain(..)
            .map(|cmd| Government::action_cost(&cmd, sim))
            .sum();

        if cost == Money::ZERO {
            return;
        }

        egui::show_tooltip(ui, Id::new("tooltip_command_cost"), |ui| {
            if cost > sim.read::<Government>().money {
                ui.colored_label(Color32::RED, format!("{cost} too expensive"));
            } else {
                ui.label(cost.to_string());
            }
        });
    }

    pub fn auto_save(&mut self, uiworld: &mut UiWorld) {
        let every = uiworld.read::<Settings>().auto_save_every.into();
        if let Some(every) = every {
            if self.last_save.elapsed() > every {
                uiworld.write::<SaveLoadState>().please_save = true;
                uiworld.save_to_disk();
                self.last_save = Instant::now();
            }
        }

        if self.last_gui_save.elapsed() > Duration::from_secs(1) {
            if let Ok(data) = common::saveload::JSONPretty::encode(self) {
                rayon::spawn(move || {
                    let _ = std::fs::write(common::saveload::JSONPretty::filename("gui"), data);
                });
            }
            self.last_gui_save = Instant::now();
        }
    }

    pub fn toolbox(ui: &Context, uiworld: &mut UiWorld, _sim: &Simulation) {
        profiling::scope!("hud::toolbox");

        if uiworld
            .read::<InputMap>()
            .just_act
            .contains(&InputAction::Close)
        {
            *uiworld.write::<Tool>() = Tool::Hand;
        }

        let [w, h]: [f32; 2] = ui.available_rect().size().into();
        //        let _tok1 = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        //        let _tok2 = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
        //        let _tok3 = ui.push_style_var(StyleVar::WindowRounding(0.0));
        //        let _tok4 = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));

        let toolbox_w = 85.0;

        let tools = [
            ("buildings", Tool::SpecialBuilding),
            ("traintool", Tool::Train),
        ];

        Window::new("Toolbox")
            .min_width(toolbox_w)
            .fixed_pos([w, h * 0.5])
            .vscroll(false)
            .frame(Frame::window(&ui.style()).rounding(0.0))
            .anchor(Align2::RIGHT_CENTER, [0.0, 0.0])
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .auto_sized()
            .show(ui, |ui| {
                let cur_tool = *uiworld.read::<Tool>();

                for (name, tool) in &tools {
                    if egui::ImageButton::new(SizedTexture::new(
                        uiworld.read::<UiTextures>().get(name),
                        [toolbox_w, 30.0],
                    ))
                    .selected(tool == &cur_tool)
                    .ui(ui)
                    .clicked()
                    {
                        uiworld.insert::<Tool>(*tool);
                    }
                }
            });

        if matches!(*uiworld.read::<Tool>(), Tool::Train) {
            let rbw = 150.0;
            Window::new("Trains")
                .fixed_size([rbw, 83.0])
                .fixed_pos([w - rbw - toolbox_w, h * 0.5 - 30.0])
                .hscroll(false)
                .title_bar(true)
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    ui.style_mut().spacing.interact_size = [rbw, 30.0].into();

                    let mut addtrain = RichText::new("Add Train");
                    if *uiworld.read::<Tool>() == Tool::Train {
                        addtrain = addtrain.strong();
                    };
                    if ui.button(addtrain).clicked() {
                        *uiworld.write::<Tool>() = Tool::Train;
                    }

                    /*
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
                            asset: "trainstation.glb".to_string(),
                            road_snap: false,
                        });
                    }*/

                    for proto in prototypes_iter::<FreightStationPrototype>() {
                        let mut freightstation = RichText::new(&proto.label);
                        if *uiworld.read::<Tool>() == Tool::SpecialBuilding {
                            freightstation = freightstation.strong();
                        };
                        if ui.button(freightstation).clicked() {
                            *uiworld.write::<Tool>() = Tool::SpecialBuilding;

                            uiworld.write::<SpecialBuildingResource>().opt =
                                Some(SpecialBuildKind {
                                    make: Box::new(move |args| {
                                        let obb = args.obb;
                                        let c = obb.center().z(args.mpos.z + 0.3);

                                        let [offx, offy] = obb.axis().map(|x| x.normalize().z(0.0));

                                        let pat = LanePatternBuilder::new()
                                            .rail(true)
                                            .one_way(true)
                                            .build();

                                        let mut commands = Vec::with_capacity(5);

                                        commands.push(WorldCommand::MapMakeConnection {
                                            from: MapProject::ground(
                                                c - offx * 45.0 - offy * 100.0,
                                            ),
                                            to: MapProject::ground(c - offx * 45.0 + offy * 100.0),
                                            inter: None,
                                            pat,
                                        });

                                        commands.push(WorldCommand::MapBuildSpecialBuilding {
                                            pos: args.obb,
                                            kind: BuildingKind::RailFreightStation(proto.id),
                                            gen: BuildingGen::NoWalkway {
                                                door_pos: Vec2::ZERO,
                                            },
                                            zone: None,
                                            connected_road: args.connected_road,
                                        });
                                        commands
                                    }),
                                    size: proto.size,
                                    asset: proto.asset_location.clone(),
                                    road_snap: false,
                                });
                        }
                    }
                });
        }

        let building_select_w = 200.0;

        if matches!(*uiworld.read::<Tool>(), Tool::SpecialBuilding) {
            Window::new("Buildings")
                .min_width(building_select_w)
                .default_height(500.0f32.min(h * 0.5))
                .vscroll(true)
                .fixed_pos([w - toolbox_w - building_select_w, h * 0.5 - 100.0])
                .title_bar(true)
                .collapsible(false)
                .resizable(false)
                .show(ui, |ui| {
                    let mut cur_build = uiworld.write::<SpecialBuildingResource>();

                    let mut picked_descr = None;
                    ui.style_mut().spacing.interact_size = [building_select_w - 5.0, 35.0].into();

                    for descr in prototypes_iter::<GoodsCompanyPrototype>() {
                        let cur_kind = cur_build.opt.as_ref().map(|x| &*x.asset).unwrap_or("");

                        let mut name = RichText::new(&descr.name);
                        if descr.asset_location == cur_kind {
                            picked_descr = Some(descr);
                            name = name.strong();
                        };
                        if ui.button(name).clicked() || cur_build.opt.is_none() {
                            let bkind = BuildingKind::GoodsCompany(descr.id);
                            let bgen = descr.bgen;
                            let has_zone = descr.zone.is_some();
                            cur_build.opt = Some(SpecialBuildKind {
                                road_snap: true,
                                make: Box::new(move |args| {
                                    vec![WorldCommand::MapBuildSpecialBuilding {
                                        pos: args.obb,
                                        kind: bkind,
                                        gen: bgen,
                                        zone: has_zone.then(|| {
                                            Zone::new(
                                                Polygon::from(args.obb.corners.as_slice()),
                                                Vec2::X,
                                            )
                                        }),
                                        connected_road: args.connected_road,
                                    }]
                                }),
                                size: descr.size,
                                asset: descr.asset_location.to_string(),
                            });
                        }
                    }

                    let bdescrpt_w = 180.0;

                    if let Some(descr) = picked_descr {
                        Window::new("Building description")
                            .default_width(bdescrpt_w)
                            .auto_sized()
                            .fixed_pos([
                                w - toolbox_w - building_select_w - bdescrpt_w,
                                h * 0.5 - 30.0,
                            ])
                            .hscroll(false)
                            .title_bar(true)
                            .collapsible(false)
                            .resizable(false)
                            .show(ui.ctx(), |ui| {
                                ui.label(format!("workers: {}", descr.n_workers));

                                if let Some(ref recipe) = descr.recipe {
                                    ui.add_space(10.0);
                                    if !recipe.consumption.is_empty() {
                                        ui.label("consumption:");
                                        for item in &recipe.consumption {
                                            item_icon(ui, uiworld, item.id, item.amount);
                                        }
                                        ui.add_space(10.0);
                                    }
                                    if !recipe.production.is_empty() {
                                        ui.label("production:");
                                        for item in &recipe.production {
                                            item_icon(ui, uiworld, item.id, item.amount);
                                        }
                                        ui.add_space(10.0);
                                    }
                                    ui.label(format!("time: {}", recipe.duration));
                                    ui.label(format!(
                                        "storage multiplier: {}",
                                        recipe.storage_multiplier
                                    ));
                                }

                                if let Some(p) = descr.power_consumption {
                                    ui.add_space(10.0);
                                    ui.label(format!("Power: {}", p));
                                }
                                if let Some(p) = descr.power_production {
                                    ui.add_space(10.0);
                                    ui.label(format!("Power production: {}", p));
                                }
                            });
                    }
                });
        }
    }

    pub fn menu_bar(&mut self, ui: &Context, uiworld: &mut UiWorld, sim: &Simulation) {
        profiling::scope!("hud::menu_bar");
        //let _t = ui.push_style_var(StyleVar::ItemSpacing([3.0, 0.0]));

        egui::TopBottomPanel::top("top_menu").show(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                self.windows.menu(ui);

                let mut name = "Save";
                let mut enabled = true;
                let mut slstate = uiworld.write::<SaveLoadState>();
                if slstate.saving_status.load(Ordering::SeqCst) {
                    name = "Saving...";
                    enabled = false;
                }

                if ui.add_enabled(enabled, egui::Button::new(name)).clicked() {
                    slstate.please_save = true;
                    self.last_save = Instant::now();
                    uiworld.save_to_disk();
                }

                ui.label(format!("Money: {}", sim.read::<Government>().money));

                let mut estate = uiworld.write::<ExitState>();

                match *estate {
                    ExitState::NoExit => {}
                    ExitState::ExitAsk | ExitState::Saving => {
                        let [w, h]: [f32; 2] = ui.available_size().into();
                        let mut opened = true;
                        Window::new("Exit Menu")
                            .default_pos([w * 0.5, h * 0.5])
                            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                            .auto_sized()
                            .open(&mut opened)
                            .show(ui.ctx(), |ui| {
                                //let _tok = ui.push_style_var(StyleVar::ItemSpacing([2.0, 5.0]));
                                if let ExitState::Saving = *estate {
                                    ui.label("Saving...");
                                    if !slstate.please_save
                                        && !slstate.saving_status.load(Ordering::SeqCst)
                                    {
                                        std::process::exit(0);
                                    }
                                    return;
                                }
                                if ui.button("Save and exit").clicked() {
                                    if let ExitState::ExitAsk = *estate {
                                        slstate.please_save = true;
                                        *estate = ExitState::Saving;
                                    }
                                }
                                if ui.button("Exit without saving").clicked() {
                                    std::process::exit(0);
                                }
                                if ui.button("Cancel").clicked() {
                                    *estate = ExitState::NoExit;
                                }
                            });
                        if !opened
                            || uiworld
                                .read::<InputMap>()
                                .just_act
                                .contains(&InputAction::Close)
                        {
                            *estate = ExitState::NoExit;
                        }
                    }
                }

                match *estate {
                    ExitState::NoExit => {
                        if ui.button("Exit").clicked() {
                            *estate = ExitState::ExitAsk;
                        }
                    }
                    ExitState::ExitAsk => {
                        if ui.button("Save and exit").clicked() {
                            if let ExitState::ExitAsk = *estate {
                                slstate.please_save = true;
                                *estate = ExitState::Saving;
                            }
                        }
                    }
                    ExitState::Saving => {
                        ui.label("Saving...");
                    }
                }
            });
        });
    }
}

pub fn item_icon(ui: &mut Ui, uiworld: &UiWorld, id: ItemID, multiplier: i32) -> Response {
    let item = id.prototype();
    ui.horizontal(move |ui| {
        if let Some(id) = uiworld
            .read::<UiTextures>()
            .try_get(&format!("icon/{}", item.name))
        {
            if ui.image(SizedTexture::new(id, (32.0, 32.0))).hovered() {
                egui::show_tooltip(ui.ctx(), ui.make_persistent_id("icon tooltip"), |ui| {
                    ui.image(SizedTexture::new(id, (64.0, 64.0)));
                    ui.label(format!("{} x{}", item.name, multiplier));
                });
            }
        } else {
            ui.label(format!("- {} ", &item.label));
        }
        ui.label(format!("x{multiplier}"))
    })
    .inner
}

pub enum ExitState {
    NoExit,
    ExitAsk,
    Saving,
}

impl Default for ExitState {
    fn default() -> Self {
        Self::NoExit
    }
}
