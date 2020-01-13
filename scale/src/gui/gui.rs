use engine::gui::imgui_wrapper::Gui;
use engine::specs::world::World;
use engine::specs::WorldExt;
use engine::systems::SelectedEntity;
use imgui::im_str;
use imgui::Ui;

#[derive(Clone)]
pub struct TestGui;

impl Gui for TestGui {
    fn render(&self, ui: &Ui, world: &mut World) {
        let selected = *world.read_resource::<SelectedEntity>();
        // Window
        imgui::Window::new(im_str!("Hello world"))
            .size([300.0, 600.0], imgui::Condition::FirstUseEver)
            .position([100.0, 100.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("こんにちは世界！"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();

                if let Some(e) = selected.0 {
                    ui.text(im_str!("Selected entity"));
                    crate::gui::inspect::InspectRenderer {
                        world,
                        entity: e,
                        ui,
                    }
                    .render();
                }

                if ui.small_button(im_str!("small button")) {
                    println!("Small button clicked");
                }
            });

        // Menu bar
        ui.main_menu_bar(|| {
            ui.menu(im_str!("Menu 1"), true, || {
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

            ui.menu(im_str!("Menu 2"), true, || {
                if imgui::MenuItem::new(im_str!("Item 2.1")).build(&ui) {
                    println!("item 2.1 inside menu bar clicked");
                }
            });
        });
    }
}
