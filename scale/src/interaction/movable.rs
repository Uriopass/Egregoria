use crate::engine_interaction::{DeltaTime, MouseButton, MouseInfo};
use crate::interaction::SelectedEntity;
use crate::physics::physics_components::{Kinematics, Transform};
use cgmath::num_traits::zero;
use cgmath::Vector2;
use imgui_inspect_derive::*;
use specs::prelude::*;
use specs::shrev::EventChannel;
use specs::Component;
use std::f32;

#[derive(Component, Default, Inspect, Clone)]
#[storage(NullStorage)]
pub struct Movable;

#[derive(Debug)]
pub struct MovedEvent {
    pub entity: Entity,
    pub new_pos: Vector2<f32>,
}

pub struct MovableSystem {
    offset: Option<Vector2<f32>>,
}

impl Default for MovableSystem {
    fn default() -> Self {
        MovableSystem { offset: None }
    }
}

impl<'a> System<'a> for MovableSystem {
    type SystemData = (
        Read<'a, MouseInfo>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        Write<'a, EventChannel<MovedEvent>>,
        ReadStorage<'a, Movable>,
        Read<'a, SelectedEntity>,
        Read<'a, DeltaTime>,
    );

    fn run(
        &mut self,
        (
            mouse,
            mut transforms,
            mut kinematics,
            mut movedevents,
            movables,
            selected,
            delta,
        ): Self::SystemData,
    ) {
        if mouse.buttons.contains(&MouseButton::Left)
            && selected.0.map_or(false, |e| movables.get(e).is_some())
        {
            let e = selected.0.unwrap();
            match self.offset {
                None => {
                    let p = transforms.get_mut(e).unwrap();
                    if let Some(kin) = kinematics.get_mut(e) {
                        kin.velocity = zero();
                        kin.acceleration = zero();
                    }
                    self.offset = Some(p.position() - mouse.unprojected);
                }
                Some(off) => {
                    let p = transforms.get_mut(e).unwrap();
                    if let Some(kin) = kinematics.get_mut(e) {
                        kin.velocity = zero();
                        kin.acceleration = zero();
                    }
                    let new_pos = off + mouse.unprojected;
                    p.set_position(new_pos);
                    movedevents.single_write(MovedEvent { entity: e, new_pos });
                }
            }
        } else if let Some(off) = self.offset.take() {
            if let Some(e) = selected.0 {
                if let Some(kin) = kinematics.get_mut(e) {
                    let p = transforms.get(e).unwrap();
                    kin.velocity = (mouse.unprojected - (p.position() - off)) / delta.0;
                }
            }
        }
    }
}
