use crate::components::{Selectable, Transform};
use crate::resources::MouseInfo;

use cgmath::InnerSpace;
use ggez::input::mouse::MouseButton;
use specs::prelude::*;
use std::f32;

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
