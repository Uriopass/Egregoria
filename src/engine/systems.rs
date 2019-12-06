use crate::engine::components::{Movable, Position};
use crate::engine::resources::MouseInfo;
use cgmath::{InnerSpace, Vector2};
use ggez::input::mouse::{MouseButton, MouseContext};
use specs::prelude::*;
use std::f32;
use std::ops::Deref;

pub struct MovableSystem {
    last_pos: Vector2<f32>,
    selected: Option<Entity>,
}

impl Default for MovableSystem {
    fn default() -> Self {
        MovableSystem {
            last_pos: [0.0, 0.0].into(),
            selected: None,
        }
    }
}

impl<'a> System<'a> for MovableSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, MouseInfo>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Movable>,
    );

    fn run(&mut self, (entities, mouse, mut pos, movables): Self::SystemData) {
        let mouse: &MouseInfo = &*mouse;

        if mouse.buttons.contains(&MouseButton::Left) {
            match self.selected {
                None => {
                    let mut min_dist: f32 = f32::MAX;
                    for (entity, pos, _) in (&entities, &pos, &movables).join() {
                        let dist: f32 = (pos.0 - mouse.unprojected).magnitude2();
                        if dist <= min_dist {
                            self.selected = Some(entity);
                            min_dist = dist;
                        }
                    }
                }
                Some(x) => {
                    let x: Entity = x;
                    let p = pos.get_mut(x);
                    p.unwrap().0 = mouse.unprojected;
                }
            }
        } else {
            self.selected = None;
        }
    }
}
