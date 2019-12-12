use cgmath::{InnerSpace, Vector2};

use specs::prelude::*;
use specs::Component;

use crate::engine::components::{CircleRender, Movable, Position, Velocity};
use crate::engine::resources::DeltaTime;
use crate::{add_shape, PhysicsWorld};

use cgmath::num_traits::zero;
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
        let mut force = -0.2 * speed.0;
        force += Vector2::unit_y() * -200.;
        return force;
        force += (self.objective - position.0).normalize() * 20.;

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
    const SCALE: f32 = 1000.;

    for _ in 0..100 {
        let size = 10.;

        let x: f32 = rand::random::<f32>() * SCALE - SCALE / 2.;
        let y: f32 = rand::random::<f32>() * SCALE + x.abs() + 50.;

        let eb = world
            .create_entity()
            .with(CircleRender {
                radius: size,
                ..Default::default()
            })
            .with(Position([x, y].into()))
            .with(Velocity([0.0, 0.0].into()))
            .with(Human {
                size,
                objective: [SCALE * 5. - x, y].into(),
            })
            .with(Movable);

        let e = eb.build();
        let shape = Ball::new(size);

        add_shape(coworld, world, e, [x, y].into(), shape);
    }

    let e1 = world
        .create_entity()
        .with(CircleRender {
            radius: 10.,
            ..Default::default()
        })
        .with(Position([0., -100.].into()))
        .with(Velocity([1000.0, 0.0].into()))
        .with(Human {
            size: 30.,
            objective: [0., 0.].into(),
        })
        .with(Movable)
        .build();

    let shape = Ball::new(10.);
    add_shape(coworld, world, e1, [0., -100.].into(), shape);
}
