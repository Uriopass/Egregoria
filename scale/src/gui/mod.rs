use crate::engine_interaction::{RenderStats, TimeInfo};
use crate::interaction::SelectedEntity;
use crate::map_model::{LanePattern, RoadGraphSynchronizeState};
use crate::transportation::{delete_transport_entity, spawn_new_transport, TransportComponent};
use imgui::im_str;
use imgui::Ui;
pub use inspect::*;
use specs::world::World;
use specs::{Entity, Join, WorldExt};

#[macro_use]
mod inspect;

#[derive(Clone)]
pub struct Gui {
    show_car_ui: bool,
    show_stats: bool,
    show_tips: bool,
    n_cars: i32,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            show_car_ui: true,
            show_stats: true,
            show_tips: false,
            n_cars: 100,
        }
    }
}

impl Gui {
    pub fn render(&mut self, ui: &Ui, world: &mut World) {
        let selected = *world.read_resource::<SelectedEntity>();
        // Window
        if let Some(e) = selected.0 {
            let mut is_open = true;
            imgui::Window::new(im_str!("Inspect"))
                .size([300.0, 300.0], imgui::Condition::FirstUseEver)
                .position([30.0, 160.0], imgui::Condition::FirstUseEver)
                .opened(&mut is_open)
                .build(&ui, || {
                    crate::gui::inspect::InspectRenderer {
                        world,
                        entity: e,
                        ui,
                    }
                    .render();
                });
            if !is_open {
                *world.write_resource::<SelectedEntity>() = SelectedEntity(None);
            }
        }

        // Menu bar
        ui.main_menu_bar(|| {
            ui.menu(im_str!("Show"), true, || {
                if imgui::MenuItem::new(im_str!("Cars")).build(&ui) {
                    self.show_car_ui = true;
                }
                if imgui::MenuItem::new(im_str!("Stats")).build(&ui) {
                    self.show_stats = true;
                }
                if imgui::MenuItem::new(im_str!("Tips")).build(&ui) {
                    self.show_tips = true;
                }
            });
            if ui.small_button(im_str!("Save")) {
                crate::transportation::save(world);
                crate::map_model::save(world);
            }
        });

        if self.show_car_ui {
            let mut opened = self.show_car_ui;
            imgui::Window::new(im_str!("Traffic"))
                .size([200.0, 140.0], imgui::Condition::FirstUseEver)
                .position([30.0, 30.0], imgui::Condition::FirstUseEver)
                .opened(&mut opened)
                .build(&ui, || {
                    ui.set_next_item_width(70.0);
                    imgui::DragInt::new(&ui, im_str!("n_cars"), &mut self.n_cars)
                        .min(1)
                        .max(1000)
                        .build();

                    ui.same_line(0.0);
                    if ui.small_button(im_str!("spawn car")) {
                        for _ in 0..self.n_cars {
                            spawn_new_transport(world);
                        }
                    }

                    if ui.small_button(im_str!("delete all cars")) {
                        let to_delete: Vec<Entity> = (
                            &world.entities(),
                            &world.read_component::<TransportComponent>(),
                        )
                            .join()
                            .map(|(e, _)| e)
                            .collect();

                        for e in to_delete {
                            delete_transport_entity(world, e);
                        }
                    }

                    let pattern = &mut world
                        .get_mut::<RoadGraphSynchronizeState>()
                        .unwrap()
                        .pattern;

                    ui.text(im_str!("Current selection: {}", pattern.name));

                    if ui.small_button(im_str!("One way")) {
                        *pattern = LanePattern::one_way(1);
                    }

                    if ui.small_button(im_str!("One way two lanes")) {
                        *pattern = LanePattern::one_way(2);
                    }

                    if ui.small_button(im_str!("Two way")) {
                        *pattern = LanePattern::two_way(1);
                    }

                    if ui.small_button(im_str!("Two way two lanes")) {
                        *pattern = LanePattern::two_way(2);
                    }
                });
            self.show_car_ui = opened;
        }

        if self.show_stats {
            let stats = world.read_resource::<RenderStats>();
            imgui::Window::new(im_str!("Stats"))
                .size([200.0, 100.0], imgui::Condition::FirstUseEver)
                .position([300.0, 50.0], imgui::Condition::FirstUseEver)
                .opened(&mut self.show_stats)
                .build(&ui, || {
                    ui.text(im_str!("Update time: {:.1}ms", stats.update_time * 1000.0));
                    ui.text(im_str!("Render time: {:.1}ms", stats.render_time * 1000.0));
                });
        }

        if self.show_tips {
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

        let time_info = world.get_mut::<TimeInfo>().unwrap();
        let [w, h] = ui.io().display_size;
        imgui::Window::new(im_str!("Time controls"))
            .size([200.0, 40.0], imgui::Condition::Always)
            .position([w / 2.0 - 100.0, h - 30.0], imgui::Condition::Always)
            .no_decoration()
            .collapsible(false)
            .resizable(false)
            .build(&ui, || {
                imgui::Slider::new(im_str!("speed"), std::ops::RangeInclusive::new(0.0, 3.0))
                    .display_format(im_str!("%.1f"))
                    .build(&ui, &mut time_info.time_speed);
            });
    }
}
