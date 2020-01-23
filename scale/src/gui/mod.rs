use crate::cars::spawn_new_car;
use crate::interaction::SelectedEntity;
use imgui::im_str;
use imgui::Ui;
use specs::world::World;
use specs::WorldExt;

pub use inspect::*;

mod inspect;

#[derive(Clone, Default)]
pub struct Gui {
    show_car_ui: bool,
}

impl Gui {
    pub fn render(&mut self, ui: &Ui, world: &mut World) {
        let selected = *world.read_resource::<SelectedEntity>();
        // Window
        if let Some(e) = selected.0 {
            let mut is_open = true;
            imgui::Window::new(im_str!("Inspect"))
                .size([200.0, 300.0], imgui::Condition::FirstUseEver)
                .position([30.0, 30.0], imgui::Condition::FirstUseEver)
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
            ui.menu(im_str!("Infos"), true, || {
                if imgui::MenuItem::new(im_str!("Cars")).build(&ui) {
                    self.show_car_ui = true;
                }
            });
        });

        if self.show_car_ui {
            imgui::Window::new(im_str!("Cars"))
                .size([200.0, 300.0], imgui::Condition::FirstUseEver)
                .position([30.0, 330.0], imgui::Condition::FirstUseEver)
                .opened(&mut true)
                .build(&ui, || {
                    if ui.small_button(im_str!("spawn car")) {
                        spawn_new_car(world);
                    }

                    if ui.small_button(im_str!("spawn 10 cars")) {
                        (0..10).for_each(|_| spawn_new_car(world));
                    }

                    if ui.small_button(im_str!("spawn 100 cars")) {
                        (0..100).for_each(|_| spawn_new_car(world));
                    }
                });
        }
    }
}
