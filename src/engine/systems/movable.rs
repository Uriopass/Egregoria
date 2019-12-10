use crate::engine::components::{Movable, Position};
use crate::engine::resources::MouseInfo;
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
        WriteStorage<'a, Position>,
        ReadStorage<'a, Movable>,
    );

    fn run(&mut self, (entities, mouse, mut pos, movables): Self::SystemData) {
        let mouse: &MouseInfo = mouse.deref();

        if mouse.buttons.contains(&MouseButton::Left) {
            match self.selected {
                None => {
                    let mut min_dist = f32::MAX;
                    for (entity, pos, _) in (&entities, &pos, &movables).join() {
                        let dist: f32 = (pos.0 - mouse.unprojected).magnitude2();
                        if dist <= min_dist {
                            self.selected = Some(entity);
                            min_dist = dist;
                        }
                    }
                    if let Some(e) = self.selected {
                        let p = pos.get_mut(e).unwrap();
                        self.offset = p.0 - mouse.unprojected;
                    }
                }
                Some(x) => {
                    let p = pos.get_mut(x).unwrap();
                    p.0 = self.offset + mouse.unprojected;
                }
            }
        } else {
            self.selected = None;
        }
    }
}
