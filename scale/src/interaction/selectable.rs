use crate::engine_interaction::KeyCode;
use crate::engine_interaction::{KeyboardInfo, MouseButton, MouseInfo};
use crate::geometry::Vec2;
use crate::interaction::Tool;
use crate::physics::Transform;
use cgmath::InnerSpace;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::shrev::EventChannel;
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

pub struct DeletedEvent {
    pub e: Entity,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct InspectedEntity {
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
        Read<'a, Tool>,
        Write<'a, InspectedEntity>,
        Write<'a, EventChannel<DeletedEvent>>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Selectable>,
    );

    fn run(
        &mut self,
        (entities, mouse, kbinfo, tool, mut inspected, mut deleted_chan, transforms, selectables): Self::SystemData,
    ) {
        if mouse.just_pressed.contains(&MouseButton::Left) && matches!(*tool, Tool::Hand) {
            inspected.e = closest_entity(
                (&entities, &transforms, &selectables).join(),
                mouse.unprojected,
            );
        }

        if let Some(e) = inspected.e {
            if !entities.is_alive(e) {
                inspected.e = None;
                return;
            }

            if kbinfo.just_pressed.contains(&KeyCode::Backspace) {
                entities.delete(e).unwrap();
                deleted_chan.single_write(DeletedEvent { e });
                inspected.e = None;
            }
        }

        if kbinfo.just_pressed.contains(&KeyCode::Escape) || matches!(*tool, Tool::Bulldozer) {
            inspected.e = None;
        }
    }
}

fn closest_entity<'a>(
    it: impl IntoIterator<Item = (Entity, &'a Transform, &'a Selectable)>,
    pos: Vec2,
) -> Option<Entity> {
    it.into_iter()
        .map(|(e, trans, select)| (e, select, (trans.position() - pos).magnitude2()))
        .filter(|(_, select, dist2)| *dist2 <= select.radius * select.radius)
        .min_by_key(|(_, _, d)| OrderedFloat(*d))
        .map(|(e, _, _)| e)
}
