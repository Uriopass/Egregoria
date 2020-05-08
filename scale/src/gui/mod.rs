use crate::engine_interaction::{MouseInfo, RenderStats, TimeInfo};
use crate::interaction::{InspectedEntity, RoadBuildState, Tool};
use crate::map_model::{LanePatternBuilder, Map};
use crate::pedestrians::{delete_pedestrian, spawn_pedestrian, PedestrianComponent};
use crate::vehicles::{delete_vehicle_entity, spawn_new_vehicle, VehicleComponent};
use imgui::Ui;
use imgui::{im_str, StyleVar};
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
pub use inspect::*;
use specs::world::World;
use specs::{Entity, Join, WorldExt};

#[macro_use]
mod inspect;

#[derive(Clone)]
pub struct Gui {
    show_map_ui: bool,
    show_info: bool,
    show_tips: bool,
    n_cars: i32,
    n_pedestrians: i32,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            show_map_ui: true,
            show_info: true,
            show_tips: false,
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

        self.toolbox(ui, world);

        self.time_controls(ui, world);
    }

    pub fn toolbox(&mut self, ui: &Ui, world: &mut World) {
        let [w, h] = ui.io().display_size;
        let tok = ui.push_style_vars(&[
            StyleVar::WindowPadding([0.0, 0.0]),
            StyleVar::WindowBorderSize(0.0),
            StyleVar::WindowRounding(0.0),
            StyleVar::ItemSpacing([0.0, 0.0]),
        ]);
        imgui::Window::new(im_str!("Toolbox"))
            .size([80.0, 110.0], imgui::Condition::Always)
            .position([w - 80.0, h / 2.0 - 30.0], imgui::Condition::Always)
            .scroll_bar(false)
            .title_bar(true)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .build(&ui, || {
                let cur_tool: &mut Tool = &mut world.write_resource::<Tool>();

                let tools = [
                    (im_str!("Hand"), Tool::Hand),
                    (im_str!("Road Build"), Tool::Roadbuild),
                    (im_str!("Bulldozer"), Tool::Bulldozer),
                ];

                for (name, tool) in &tools {
                    let tok = ui.push_style_var(StyleVar::Alpha(if tool == cur_tool {
                        1.0
                    } else {
                        0.5
                    }));
                    if ui.button(name, [80.0, 30.0]) {
                        *cur_tool = *tool;
                    }
                    tok.pop(ui);
                }
            });
        tok.pop(ui);
    }

    pub fn inspector(&mut self, ui: &Ui, world: &mut World) {
        let mut inspected = *world.read_resource::<InspectedEntity>();
        let e = unwrap_or!(inspected.e, return);

        let mut is_open = true;
        imgui::Window::new(im_str!("Inspect"))
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
        let time_info = world.get_mut::<TimeInfo>().unwrap();
        let [w, h] = ui.io().display_size;
        imgui::Window::new(im_str!("Time controls"))
            .size([230.0, 40.0], imgui::Condition::Always)
            .position([w / 2.0 - 100.0, h - 30.0], imgui::Condition::Always)
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
        if !self.show_info {
            return;
        }
        let stats = world.read_resource::<RenderStats>();
        let mouse = world.read_resource::<MouseInfo>().unprojected;
        imgui::Window::new(im_str!("Info"))
            .size([200.0, 100.0], imgui::Condition::FirstUseEver)
            .position([300.0, 50.0], imgui::Condition::FirstUseEver)
            .opened(&mut self.show_info)
            .build(&ui, || {
                ui.text(im_str!("Update time: {:.1}ms", stats.update_time * 1000.0));
                ui.text(im_str!("Render time: {:.1}ms", stats.render_time * 1000.0));
                ui.text(im_str!("Mouse pos: {:.1} {:.1}", mouse.x, mouse.y));
            });
    }

    pub fn tips(&mut self, ui: &Ui) {
        if !self.show_tips {
            return;
        }
        imgui::Window::new(im_str!("Tips"))
            .size([280.0, 200.0], imgui::Condition::FirstUseEver)
            .position([30.0, 470.0], imgui::Condition::FirstUseEver)
            .opened(&mut self.show_tips)
            .build(&ui, || {
                ui.text(im_str!("Select: Left click"));
                ui.text(im_str!("Move: Left drag"));
                ui.text(im_str!("Deselect: Escape"));
                ui.text(im_str!("Pan: Right click or Arrow keys"));
                ui.separator();
                ui.text(im_str!("Add intersection: I"));
                ui.text(im_str!("Connect intersections: C"));
                ui.text(im_str!("Disconnect intersections: C"));
                ui.text(im_str!("Delete intersection: Backspace"));
            });
    }

    pub fn menu_bar(&mut self, ui: &Ui, world: &mut World) {
        ui.main_menu_bar(|| {
            ui.menu(im_str!("Show"), true, || {
                if imgui::MenuItem::new(im_str!("Map")).build(&ui) {
                    self.show_map_ui = true;
                }
                if imgui::MenuItem::new(im_str!("Info")).build(&ui) {
                    self.show_info = true;
                }
                if imgui::MenuItem::new(im_str!("Tips")).build(&ui) {
                    self.show_tips = true;
                }
            });
            if ui.small_button(im_str!("Save")) {
                crate::vehicles::save(world);
                crate::map_model::save(world);
            }
        });
    }

    pub fn map_ui(&mut self, ui: &Ui, world: &mut World) {
        if !self.show_map_ui {
            return;
        }

        let mut opened = self.show_map_ui;
        imgui::Window::new(im_str!("Map"))
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
                        spawn_new_vehicle(world);
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

                if ui.small_button(im_str!("delete all cars")) {
                    let to_delete: Vec<Entity> = (
                        &world.entities(),
                        &world.read_component::<VehicleComponent>(),
                    )
                        .join()
                        .map(|(e, _)| e)
                        .collect();

                    for e in to_delete {
                        delete_vehicle_entity(world, e);
                    }
                }

                if ui.small_button(im_str!("delete all pedestrians")) {
                    let to_delete: Vec<Entity> = (
                        &world.entities(),
                        &world.read_component::<PedestrianComponent>(),
                    )
                        .join()
                        .map(|(e, _)| e)
                        .collect();

                    for e in to_delete {
                        delete_pedestrian(world, e);
                    }
                }

                let mut pattern = world.get_mut::<RoadBuildState>().unwrap().pattern_builder;

                <LanePatternBuilder as InspectRenderDefault<LanePatternBuilder>>::render_mut(
                    &mut [&mut pattern],
                    "Road shape",
                    world,
                    &ui,
                    &InspectArgsDefault::default(),
                );

                if pattern.n_lanes == 0 {
                    pattern.sidewalks = true;
                }

                world.write_resource::<RoadBuildState>().pattern_builder = pattern;

                let map: &mut Map = &mut world.write_resource::<Map>();

                if ui.small_button(im_str!("load Paris map")) {
                    map.clear();
                    crate::map_model::load_parismap(map);
                }

                if ui.small_button(im_str!("load test field")) {
                    map.clear();
                    crate::map_model::load_testfield(map);
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
