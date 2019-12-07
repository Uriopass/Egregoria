use cgmath::{InnerSpace, Vector2};
use ggez::graphics::{Color, WHITE};

use specs::prelude::*;
use specs::Component;

use crate::engine::components::{CircleRender, Movable, Position, Velocity};
use crate::engine::resources::DeltaTime;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Human {
    direction: Vector2<f32>,
    size: f32,
    objective: Vector2<f32>,
}

impl Human {
    fn calc_acceleration(
        &self,
        position: &Position,
        speed: &Velocity,
        others: &Vec<(&Position, &Human)>,
    ) -> Vector2<f32> {
        let mut force: Vector2<f32> = (self.objective - position.0) * 0.3;

        force -= speed.0;

        for (p, h) in others {
            let mut x: Vector2<f32> = position.0 - p.0;
            if x.x == 0. && x.y == 0. {
                continue;
            }
            x *= h.size * h.size * 0.5 / x.magnitude2();
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
            let acc = h.calc_acceleration(&p, &v, &xx);
            v.0 += acc * delta * 2.;
        })
    }
}

pub fn setup(world: &mut World) {
    for _ in 0..5000 {
        let r: f32 = rand::random();
        let r = 20. + r * 20.;
        world
            .create_entity()
            .with(CircleRender {
                radius: r,
                color: WHITE,
            })
            .with(Position(
                [
                    rand::random::<f32>() * 1000. - 500.,
                    rand::random::<f32>() * 1000. - 500.,
                ]
                .into(),
            ))
            .with(Velocity([0.0, 1.0].into()))
            .with(Human {
                direction: [1.0, 0.0].into(),
                size: r,
                objective: [0.0, 0.0].into(),
            })
            .build();
    }

    world
        .create_entity()
        .with(CircleRender {
            radius: 200.,
            color: ggez::graphics::Color::new(1., 0., 0., 1.),
        })
        .with(Position(
            [
                rand::random::<f32>() * 1000. - 500.,
                rand::random::<f32>() * 1000. - 500.,
            ]
            .into(),
        ))
        .with(Velocity([0.0, 1.0].into()))
        .with(Human {
            direction: [1.0, 0.0].into(),
            size: 2000.,
            objective: [0.0, 0.0].into(),
        })
        .with(Movable)
        .build();
}
