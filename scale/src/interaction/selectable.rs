use crate::engine_interaction::KeyCode;
use crate::engine_interaction::{KeyboardInfo, MouseButton, MouseInfo};
use crate::physics::physics_components::Transform;
use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::Component;
use std::f32;

#[derive(Component, Default, Clone, Serialize, Deserialize)]
#[storage(NullStorage)]
pub struct Selectable;

#[derive(Default, Clone, Copy)]
pub struct SelectedEntity(pub Option<Entity>);

pub struct SelectableSystem;
impl<'a> System<'a> for SelectableSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, MouseInfo>,
        Read<'a, KeyboardInfo>,
        Write<'a, SelectedEntity>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Selectable>,
    );

    fn run(
        &mut self,
        (entities, mouse, kbinfo, mut selected, transforms, selectables): Self::SystemData,
    ) {
        if mouse.just_pressed.contains(&MouseButton::Left) {
            let mut min_dist = f32::MAX;
            let mut closest = None;
            for (entity, trans, _) in (&entities, &transforms, &selectables).join() {
                let dist: f32 = (trans.position() - mouse.unprojected).magnitude2();
                if dist <= min_dist {
                    closest = Some(entity);
                    min_dist = dist;
                }
            }
            *selected = SelectedEntity(closest);
        }

        if kbinfo.just_pressed.contains(&KeyCode::Escape) {
            *selected = SelectedEntity(None);
        }
    }
}
