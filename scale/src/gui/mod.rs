mod inspect;

pub use inspect::*;

use crate::engine_interaction::{MouseButton, MouseInfo};
use crate::interaction::SelectedEntity;
use imgui::im_str;
use imgui::Ui;
use specs::world::World;
use specs::WorldExt;

#[derive(Clone)]
pub struct TestGui;

impl TestGui {
    pub fn render(&self, ui: &Ui, world: &mut World) {
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

        //if world.read_resource::<MouseInfo>().just_pressed.contains(MouseButton::Right)

        // Menu bar
        ui.main_menu_bar(|| {
            ui.menu(im_str!("Physics"), true, || {
                if imgui::MenuItem::new(im_str!("Item 1.1")).build(&ui) {
                    println!("item 1.1 inside menu bar clicked");
                }

                ui.menu(im_str!("Item 1.2"), true, || {
                    if imgui::MenuItem::new(im_str!("Item 1.2.1")).build(&ui) {
                        println!("item 1.2.1 inside menu bar clicked");
                    }
                    if imgui::MenuItem::new(im_str!("Item 1.2.2")).build(&ui) {
                        println!("item 1.2.2 inside menu bar clicked");
                    }
                });
            });
        });
    }
}
