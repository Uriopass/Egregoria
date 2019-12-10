use cgmath::{InnerSpace, Vector2};
use ggez::graphics::WHITE;

use specs::prelude::*;
use specs::Component;

use crate::engine::components::{CircleRender, Movable, Position, Velocity};
use crate::engine::resources::DeltaTime;
use crate::{add_shape, PhysicsWorld};

use ncollide2d::shape::Ball;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Human {
    size: f32,
    objective: Vector2<f32>,
}

impl Human {
    fn calc_acceleration(
        &self,
        position: &Position,
        speed: &Velocity,
        others: &[(&Position, &Human)],
    ) -> Vector2<f32> {
        let mut force: Vector2<f32> = (self.objective - position.0).normalize() * 20.;

        force -= speed.0;

        for (p, h) in others {
            let mut x: Vector2<f32> = position.0 - p.0;
            if x.x == 0. && x.y == 0. {
                continue;
            }
            let d = x.magnitude();
            if d > 200. {
                continue;
            }
            x *= (h.size * self.size) / (d * d);
            force += x;
        }
        force
    }
}

pub struct HumanUpdate;

impl<'a> System<'a> for HumanUpdate {
    type SystemData = (
        Read<'a, DeltaTime>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        ReadStorage<'a, Human>,
    );

    fn run(&mut self, (delta, pos, mut vel, humans): Self::SystemData) {
        let delta = delta.0;

        let xx: Vec<(&Position, &Human)> = (&pos, &humans).join().collect();

        (&pos, &mut vel, &humans).par_join().for_each(|(p, v, h)| {
            if (h.objective - p.0).magnitude2() < 1. {
                v.0 = [0.0, 0.0].into();
                return;
            }

            let acc = h.calc_acceleration(&p, &v, &xx);
            v.0 += acc * delta * 2.;
        })
    }
}

pub fn setup(world: &mut World, coworld: &mut PhysicsWorld) {
    let mut last: Option<Entity> = None;
    const SCALE: f32 = 500.;

    for _ in 0..100 {
        let size = 10.;

        let x: f32 = if rand::random() {
            rand::random::<f32>() * SCALE
        } else {
            SCALE * 5. + rand::random::<f32>() * SCALE
        };
        let y: f32 = rand::random::<f32>() * SCALE;

        let eb = world
            .create_entity()
            .with(CircleRender {
                radius: size,
                color: WHITE,
            })
            .with(Position([x, y].into()))
            .with(Velocity([0.0, 1.0].into()))
            .with(Human {
                size,
                objective: [SCALE * 5. - x, y].into(),
            })
            .with(Movable);
        /*
        if let Some(x) = last {
            y = y.with(LineRender {
                color: ggez::graphics::WHITE,
                to: x,
            });
        }
        */

        let e = eb.build();
        let shape = Ball::new(size);

        add_shape(coworld, world, e, [x, y].into(), shape);

        last = Some(e);
    }
}
