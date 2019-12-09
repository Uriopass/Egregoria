use cgmath::{InnerSpace, Vector2};
use ggez::graphics::WHITE;

use specs::prelude::*;
use specs::Component;

use crate::engine::components::{CircleRender, LineRender, Movable, Position, Velocity};
use crate::engine::resources::DeltaTime;

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
        others: &Vec<(&Position, &Human)>,
    ) -> Vector2<f32> {
        if (self.objective - position.0).magnitude2() < 1. {
            return [0.0, 0.0].into();
        }
        let mut force: Vector2<f32> = (self.objective - position.0).normalize() * 20.;

        force -= speed.0;

        for (p, h) in others {
            let mut x: Vector2<f32> = position.0 - p.0;
            if x.x == 0. && x.y == 0. {
                continue;
            }
            let d = x.magnitude();
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
            let acc = h.calc_acceleration(&p, &v, &xx);
            v.0 += acc * delta * 2.;
        })
    }
}

pub fn setup(world: &mut World) {
    let mut last: Option<Entity> = None;
    for _ in 0..100 {
        let size: f32 = rand::random();
        let size = 10.;

        let x: f32 = if rand::random() {
            rand::random::<f32>() * 1000.
        } else {
            5000. + rand::random::<f32>() * 1000.
        };
        let y: f32 = rand::random::<f32>() * 1000.;
        println!("{}", y);

        let mut y = world
            .create_entity()
            .with(CircleRender {
                radius: size,
                color: WHITE,
            })
            .with(Position([x, y].into()))
            .with(Velocity([0.0, 1.0].into()))
            .with(Human {
                size: size * 2.,
                objective: [5000. - x, y].into(),
            });
        if let Some(x) = last {
            y = y.with(LineRender {
                color: ggez::graphics::WHITE,
                to: x,
            });
        }

        let e = y.build();
        last = Some(e);
    }
}
