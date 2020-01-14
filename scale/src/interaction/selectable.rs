use crate::empty_struct_inspect_impl;
use crate::engine_interaction::{MouseButton, MouseInfo};
use crate::physics::physics_components::Transform;
use cgmath::InnerSpace;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use specs::prelude::*;
use specs::Component;
use std::f32;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct Selectable;
empty_struct_inspect_impl!(Selectable);

#[derive(Default, Clone, Copy)]
pub struct SelectedEntity(pub Option<Entity>);

#[derive(Default)]
pub struct SelectableSystem;

impl<'a> System<'a> for SelectableSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, MouseInfo>,
        Write<'a, SelectedEntity>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Selectable>,
    );

    fn run(&mut self, (entities, mouse, mut selected, transforms, selectables): Self::SystemData) {
        if mouse.just_pressed.contains(&MouseButton::Left) {
            let mut min_dist = f32::MAX;
            let mut closest = None;
            for (entity, trans, _) in (&entities, &transforms, &selectables).join() {
                let dist: f32 = (trans.get_position() - mouse.unprojected).magnitude2();
                if dist <= min_dist {
                    closest = Some(entity);
                    min_dist = dist;
                }
            }
            *selected = SelectedEntity(closest);
            println!("Selected new entity");
        }
    }
}
