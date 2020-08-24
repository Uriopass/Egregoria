use crate::engine_interaction::{MouseInfo, RenderStats, TimeInfo};
use crate::frame_log::FrameLog;
use crate::interaction::{InspectedEntity, RoadBuildResource, Tool};
use crate::pedestrians::{spawn_pedestrian, PedestrianComponent};
use crate::utils::delete_entity;
use crate::vehicles::{spawn_parked_vehicle, VehicleComponent};
use imgui::{im_str, StyleVar};
use imgui::{Ui, Window};
use imgui_inspect::{InspectArgsStruct, InspectRenderStruct};
pub use inspect::*;
use map_model::{LanePatternBuilder, Map};
use serde::{Deserialize, Serialize};
use specs::world::World;
use specs::{Entity, Join, WorldExt};
use std::time::{Duration, Instant};

#[macro_use]
mod inspect;

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

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Gui {
    pub show_map_ui: bool,
    pub show_debug_info: bool,
    pub show_tips: bool,
    pub show_debug_layers: bool,
    pub show_scenarios: bool,
    pub auto_save_every: AutoSaveEvery,
    #[serde(skip)]
    pub last_save: Instant,
    #[serde(skip)]
    pub available_scenarios: Vec<String>,
    pub n_cars: i32,
    pub n_pedestrians: i32,
}

fn available_scenarios() -> Vec<String> {
    let mut available_scenarios = vec![];
    for file in std::fs::read_dir("lua/scenarios")
        .into_iter()
        .flatten()
        .filter_map(|x| x.ok())
    {
        available_scenarios.push(file.file_name().to_string_lossy().into_owned());
    }
    available_scenarios
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            show_map_ui: true,
            show_debug_info: false,
            show_tips: false,
            show_debug_layers: false,
            show_scenarios: false,
            auto_save_every: AutoSaveEvery::OneMinute,
            last_save: Instant::now(),
            available_scenarios: available_scenarios(),
            n_cars: 100,
            n_pedestrians: 100,
        }
    }
}

impl Gui {
    pub fn render(&mut self, ui: &Ui, world: &mut World) {
        self.inspector(ui, world);

        self.menu_bar(ui, world);

        self.map_ui(ui, world);

        self.info(ui, world);

        self.tips(ui);

        self.toolbox(ui, world);

        self.time_controls(ui, world);

        self.auto_save(world);

        self.scenario(ui, world);
    }

    pub fn scenario(&mut self, ui: &Ui, world: &mut World) {
        if !self.show_scenarios {
            return;
        }
        let scenarios = &mut self.available_scenarios;
        Window::new(im_str!("Scenarios"))
            .position([300.0, 300.0], imgui::Condition::FirstUseEver)
            .opened(&mut self.show_scenarios)
            .build(&ui, || {
                for scenario in scenarios.iter() {
                    if ui.small_button(&im_str!("{}", scenario)) {
                        crate::lua::scenario_runner::set_scenario(
                            world,
                            &format!("lua/scenarios/{}", scenario),
                        );
                    }
                }
                if ui.small_button(im_str!("reload scenario list")) {
                    *scenarios = available_scenarios();
                }
            });
    }

    pub fn auto_save(&mut self, world: &mut World) {
        if let Some(every) = self.auto_save_every.into() {
            let now = Instant::now();
            if now.duration_since(self.last_save) > every {
                crate::save_to_disk(world);
                self.last_save = now;
            }
        }
    }

    pub fn toolbox(&mut self, ui: &Ui, world: &mut World) {
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
                let cur_tool: &mut Tool = &mut world.write_resource::<Tool>();

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
            *world.read_resource::<Tool>(),
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
                    let mut pattern = world.write_resource::<RoadBuildResource>().pattern_builder;

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

                    world.write_resource::<RoadBuildResource>().pattern_builder = pattern;
                });
        }

        tok.pop(ui);
    }

    pub fn inspector(&mut self, ui: &Ui, world: &mut World) {
        let mut inspected = *world.read_resource::<InspectedEntity>();
        let e = unwrap_or!(inspected.e, return);

        let mut is_open = true;
        Window::new(im_str!("Inspect"))
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([30.0, 160.0], imgui::Condition::FirstUseEver)
            .opened(&mut is_open)
            .build(&ui, || {
                inspected.dirty =
                    crate::gui::inspect::InspectRenderer { entity: e }.render(world, ui);
            });
        if !is_open {
            inspected.e = None;
            inspected.dirty = false;
        }
        *world.write_resource::<InspectedEntity>() = inspected;
    }

    pub fn time_controls(&mut self, ui: &Ui, world: &mut World) {
        let mut time_info = world.write_resource::<TimeInfo>();
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

    pub fn info(&mut self, ui: &Ui, world: &mut World) {
        if !self.show_debug_info {
            return;
        }
        let stats = world.read_resource::<RenderStats>();
        let mouse = world.read_resource::<MouseInfo>().unprojected;
        Window::new(im_str!("Debug Info"))
            .position([300.0, 50.0], imgui::Condition::FirstUseEver)
            .opened(&mut self.show_debug_info)
            .build(&ui, || {
                ui.text(im_str!("Update time: {:.1}ms", stats.update_time * 1000.0));
                ui.text(im_str!("Render time: {:.1}ms", stats.render_time * 1000.0));
                ui.text(im_str!("Mouse pos: {:.1} {:.1}", mouse.x, mouse.y));
                ui.separator();
                ui.text("Frame log");
                let flog = world.read_resource::<FrameLog>();
                let fl = flog.get_frame_log();
                for s in &*fl {
                    ui.text(im_str!("{}", s));
                }
            });
    }

    pub fn tips(&mut self, ui: &Ui) {
        if !self.show_tips {
            return;
        }
        Window::new(im_str!("Tips"))
            .size([280.0, 200.0], imgui::Condition::FirstUseEver)
            .position([30.0, 470.0], imgui::Condition::FirstUseEver)
            .opened(&mut self.show_tips)
            .build(&ui, || {
                ui.text(im_str!("Select: Left click"));
                ui.text(im_str!("Move: Left drag"));
                ui.text(im_str!("Deselect: Escape"));
                ui.text(im_str!("Pan: Right click or Arrow keys"));
            });
    }

    pub fn menu_bar(&mut self, ui: &Ui, world: &mut World) {
        ui.main_menu_bar(|| {
            ui.menu(im_str!("Show"), true, || {
                if imgui::MenuItem::new(im_str!("Map")).build(&ui) {
                    self.show_map_ui = true;
                }
                if imgui::MenuItem::new(im_str!("Tips")).build(&ui) {
                    self.show_tips = true;
                }
                if imgui::MenuItem::new(im_str!("Scenarios")).build(&ui) {
                    self.show_scenarios = true;
                }
                if imgui::MenuItem::new(im_str!("Debug Info")).build(&ui) {
                    self.show_debug_info = true;
                }
                if imgui::MenuItem::new(im_str!("Debug Layers")).build(&ui) {
                    self.show_debug_layers = true;
                }
            });
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
                crate::save_to_disk(world);
            }
        });
    }

    pub fn map_ui(&mut self, ui: &Ui, world: &mut World) {
        if !self.show_map_ui {
            return;
        }

        let mut opened = self.show_map_ui;
        Window::new(im_str!("Map"))
            .size([200.0, 140.0], imgui::Condition::FirstUseEver)
            .position([30.0, 30.0], imgui::Condition::FirstUseEver)
            .opened(&mut opened)
            .build(&ui, || {
                ui.set_next_item_width(70.0);
                imgui::DragInt::new(&ui, im_str!("n cars"), &mut self.n_cars)
                    .min(1)
                    .max(1000)
                    .build();

                ui.same_line(0.0);
                if ui.small_button(im_str!("spawn cars")) {
                    for _ in 0..self.n_cars {
                        spawn_parked_vehicle(world);
                    }
                }

                ui.set_next_item_width(70.0);
                imgui::DragInt::new(&ui, im_str!("n pedestrians"), &mut self.n_pedestrians)
                    .min(1)
                    .max(1000)
                    .build();

                ui.same_line(0.0);
                if ui.small_button(im_str!("spawn pedestrians")) {
                    for _ in 0..self.n_pedestrians {
                        spawn_pedestrian(world);
                    }
                }

                if ui.small_button(im_str!("destroy all cars")) {
                    let to_delete: Vec<Entity> = (
                        &world.entities(),
                        &world.read_component::<VehicleComponent>(),
                    )
                        .join()
                        .map(|(e, _)| e)
                        .collect();

                    for e in to_delete {
                        delete_entity(world, e);
                    }
                }

                if ui.small_button(im_str!("kill all pedestrians")) {
                    let to_delete: Vec<Entity> = (
                        &world.entities(),
                        &world.read_component::<PedestrianComponent>(),
                    )
                        .join()
                        .map(|(e, _)| e)
                        .collect();

                    for e in to_delete {
                        delete_entity(world, e);
                    }
                }

                let map: &mut Map = &mut world.write_resource::<Map>();

                if ui.small_button(im_str!("build houses")) {
                    map.build_houses();
                }

                if ui.small_button(im_str!("load Paris map")) {
                    map.clear();
                    map_model::load_parismap(map);
                }

                if ui.small_button(im_str!("load test field")) {
                    map.clear();
                    map_model::load_testfield(map);
                }

                if ui.small_button(im_str!("clear the map")) {
                    map.clear();
                }

                ui.text(im_str!(
                    "{} pedestrians",
                    world.read_component::<PedestrianComponent>().join().count()
                ));
                ui.text(im_str!(
                    "{} vehicles",
                    world.read_component::<VehicleComponent>().join().count()
                ));
            });
        self.show_map_ui = opened;
    }
}
