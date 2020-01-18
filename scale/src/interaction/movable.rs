use crate::engine_interaction::{DeltaTime, MouseButton, MouseInfo};
use crate::physics::physics_components::{Kinematics, Transform};
use cgmath::num_traits::zero;
use cgmath::{InnerSpace, Vector2, Zero};
use imgui_inspect_derive::*;
use specs::prelude::*;
use specs::shrev::EventChannel;
use specs::Component;
use std::f32;
use std::ops::Deref;

#[derive(Component, Default, Inspect, Clone)]
#[storage(NullStorage)]
pub struct Movable;

#[derive(Debug)]
pub struct MovedEvent {
    pub entity: Entity,
    pub new_pos: Vector2<f32>,
}

pub struct MovableSystem {
    offset: Vector2<f32>,
    selected: Option<Entity>,
}

impl Default for MovableSystem {
    fn default() -> Self {
        MovableSystem {
            offset: Vector2::zero(),
            selected: None,
        }
    }
}

impl<'a> System<'a> for MovableSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, MouseInfo>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        Write<'a, EventChannel<MovedEvent>>,
        ReadStorage<'a, Movable>,
        Read<'a, DeltaTime>,
    );

    fn run(
        &mut self,
        (entities, mouse, mut transforms, mut kinematics, mut movedevents, movables, delta): Self::SystemData,
    ) {
        let mouse: &MouseInfo = mouse.deref();
        if mouse.buttons.contains(&MouseButton::Left) {
            match self.selected {
                None => {
                    let mut min_dist = f32::MAX;
                    for (entity, trans, _) in (&entities, &transforms, &movables).join() {
                        let dist: f32 = (trans.position() - mouse.unprojected).magnitude2();
                        if dist <= min_dist {
                            self.selected = Some(entity);
                            min_dist = dist;
                        }
                    }
                    if let Some(e) = self.selected {
                        let p = transforms.get_mut(e).unwrap();
                        if let Some(kin) = kinematics.get_mut(e) {
                            kin.velocity = zero();
                            kin.acceleration = zero();
                        }
                        self.offset = p.position() - mouse.unprojected;
                    }
                }
                Some(e) => {
                    let p = transforms.get_mut(e).unwrap();
                    if let Some(kin) = kinematics.get_mut(e) {
                        kin.velocity = zero();
                        kin.acceleration = zero();
                    }
                    let new_pos = self.offset + mouse.unprojected;
                    p.set_position(new_pos);
                    movedevents.single_write(MovedEvent { entity: e, new_pos });
                }
            }
        } else if let Some(e) = self.selected.take() {
            if let Some(kin) = kinematics.get_mut(e) {
                let p = transforms.get(e).unwrap();
                kin.velocity = (mouse.unprojected - (p.position() - self.offset)) / delta.0;
            }
        }
    }
}
