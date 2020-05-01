use crate::engine_interaction::KeyCode;
use crate::engine_interaction::{KeyboardInfo, MouseButton, MouseInfo};
use crate::physics::Transform;
use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::Component;
use std::f32;

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Selectable {
    pub radius: f32,
}

impl Selectable {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Default for Selectable {
    fn default() -> Self {
        Self { radius: 5.0 }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SelectedEntity {
    pub e: Option<Entity>,
    pub dirty: bool, // Modified by inspection
}

pub struct SelectableSystem;
impl<'a> System<'a> for SelectableSystem {
    #[allow(clippy::type_complexity)]
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
            let mut min_dist2 = f32::MAX;
            let mut closest = None;
            for (entity, trans, select) in (&entities, &transforms, &selectables).join() {
                let dist2: f32 = (trans.position() - mouse.unprojected).magnitude2();
                if dist2 <= min_dist2 && dist2 <= select.radius * select.radius {
                    closest = Some(entity);
                    min_dist2 = dist2;
                }
            }
            selected.e = closest;
        }

        if let Some(x) = selected.e {
            if !entities.is_alive(x) {
                selected.e = None;
            }
        }
        if kbinfo.just_pressed.contains(&KeyCode::Escape) {
            selected.e = None;
        }
    }
}
