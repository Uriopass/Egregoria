use crate::components::{Kinematics, Movable, Transform};
use crate::resources::{DeltaTime, MouseInfo};

use crate::PHYSICS_UPDATES;
use cgmath::num_traits::zero;
use cgmath::{InnerSpace, Vector2, Zero};
use ggez::input::mouse::MouseButton;
use specs::prelude::*;
use std::f32;
use std::ops::Deref;

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
        ReadStorage<'a, Movable>,
        Read<'a, DeltaTime>,
    );

    fn run(
        &mut self,
        (entities, mouse, mut transforms, mut kinematics, movables, delta): Self::SystemData,
    ) {
        let mouse: &MouseInfo = mouse.deref();

        if mouse.buttons.contains(&MouseButton::Left) {
            match self.selected {
                None => {
                    let mut min_dist = f32::MAX;
                    for (entity, pos, _) in (&entities, &transforms, &movables).join() {
                        let dist: f32 = (pos.get_position() - mouse.unprojected).magnitude2();
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
                        self.offset = p.get_position() - mouse.unprojected;
                    }
                }
                Some(e) => {
                    let p = transforms.get_mut(e).unwrap();
                    if let Some(kin) = kinematics.get_mut(e) {
                        kin.velocity = zero();
                        kin.acceleration = zero();
                    }
                    p.set_position(self.offset + mouse.unprojected);
                }
            }
        } else if let Some(e) = self.selected.take() {
            if let Some(kin) = kinematics.get_mut(e) {
                let p = transforms.get(e).unwrap();
                kin.velocity = (mouse.unprojected - (p.get_position() - self.offset))
                    / (PHYSICS_UPDATES as f32 * delta.0);
            }
        }
    }
}
