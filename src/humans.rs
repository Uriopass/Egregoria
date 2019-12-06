use cgmath::{InnerSpace, Vector2};
use ggez::graphics::Color;

use legion::prelude::*;

use crate::engine::components::{CircleRender, Position, Velocity};
use crate::engine::resources::DeltaTime;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Human {
    direction: Vector2<f32>,
    size: f32,
    objective: Vector2<f32>,
    color: Color,
}

impl Human {
    fn calc_acceleration(
        &self,
        position: Position,
        speed: Velocity,
        others: &Vec<(Vector2<f32>, f32)>,
    ) -> Vector2<f32> {
        let mut force: Vector2<f32> = (self.objective - position.0) * 0.3;

        force -= speed.0;

        for (p, size) in others.iter() {
            let mut x: Vector2<f32> = position.0 - p;
            if x.x == 0. && x.y == 0. {
                continue;
            }
            x *= size * size * 0.5 / x.magnitude2();
            force += x;
        }
        force
    }
}

pub fn setup(world: &mut World) -> Box<dyn Schedulable> {
    world.insert(
        (),
        (0..100).map(|_i| {
            let r: f32 = rand::random();
            let r = 15. + r * 100.;
            (
                Position(
                    [
                        rand::random::<f32>() * 1000. - 500.,
                        rand::random::<f32>() * 1000. - 500.,
                    ]
                    .into(),
                ),
                Velocity([0.0, 1.0].into()),
                Human {
                    direction: [1.0, 0.0].into(),
                    size: r,
                    objective: [0.0, 0.0].into(),
                    color: ggez::graphics::WHITE,
                },
                CircleRender { radius: r },
            )
        }),
    );

    SystemBuilder::new("h up")
        .with_query(<(Read<Position>, Write<Velocity>, Read<Human>)>::query())
        .read_resource::<DeltaTime>()
        .build(|_, mut w, resources, query| {
            let delta: f32 = (**resources).0;
            let x: Vec<(Vector2<f32>, f32)> =
                query.iter(&mut w).map(|(p, _, h)| (p.0, h.size)).collect();

            for (pos, mut vel, h) in query.iter(&mut w) {
                let h: Human = *h;
                let acc = h.calc_acceleration(*pos, *vel, &x);
                vel.0 += acc * delta * 2.;
            }
        })
}
