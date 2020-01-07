#![windows_subsystem = "windows"]

use engine::ncollide2d::world::CollisionWorld;
use engine::specs::{DispatcherBuilder, World, WorldExt};

use crate::cars::car_graph::RoadGraphSynchronize;
use crate::cars::car_system::CarDecision;
use crate::humans::HumanUpdate;
use engine::components::{Collider, MeshRenderComponent};
use engine::gui::imgui_wrapper::Gui;
use engine::resources::{DeltaTime, KeyboardInfo};
use engine::systems::{KinematicsApply, MovableSystem, PhysicsUpdate};
use engine::PhysicsWorld;
use imgui::im_str;
use imgui::Ui;

mod cars;
mod graphs;
mod humans;

#[derive(Clone)]
struct TestGui;
impl Gui for TestGui {
    fn render(&self, ui: &Ui, world: &mut World) {
        // Various ui things
        {
            // Window
            imgui::Window::new(im_str!("Hello world"))
                .size([300.0, 600.0], imgui::Condition::FirstUseEver)
                .position([100.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(im_str!("Hello world!"));
                    ui.text(im_str!("こんにちは世界！"));
                    ui.text(im_str!("This...is...imgui-rs!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0],
                        mouse_pos[1]
                    ));

                    if ui.small_button(im_str!("small button")) {
                        println!("Small button clicked");
                    }
                });

            // Popup
            ui.popup(im_str!("popup"), || {
                if imgui::MenuItem::new(im_str!("popup menu item 1")).build(&ui) {
                    println!("popup menu item 1 clicked");
                }

                if imgui::MenuItem::new(im_str!("popup menu item 2")).build(&ui) {
                    println!("popup menu item 2 clicked");
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
}

fn main() {
    let collision_world: PhysicsWorld = CollisionWorld::new(2.0);

    let mut world = World::new();

    world.insert(DeltaTime(0.0));
    world.insert(collision_world);
    world.insert(KeyboardInfo::default());
    world.insert(TestGui);

    world.register::<MeshRenderComponent>();
    world.register::<Collider>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human update", &[])
        .with(CarDecision, "car decision", &[])
        .with(
            MovableSystem::default(),
            "movable",
            &["human update", "car decision"],
        )
        .with(RoadGraphSynchronize::new(&mut world), "rgs", &["movable"])
        .with(KinematicsApply, "speed apply", &["movable"])
        .with(PhysicsUpdate::default(), "physics", &["speed apply"])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world);
    cars::setup(&mut world);

    engine::start::<TestGui>(world, dispatcher);
}
