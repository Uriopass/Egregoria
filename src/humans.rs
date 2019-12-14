use cgmath::{InnerSpace, Vector2};

use specs::{
    Builder, Component, Join, ParJoin, Read, ReadStorage, System, VecStorage, World, WorldExt,
    WriteStorage,
};

use crate::add_shape;
use crate::engine::components::{
    Kinematics, LineRender, MeshRenderComponent, MeshRenderable, Movable, Position, RectRender,
};
use crate::engine::resources::DeltaTime;

use ncollide2d::shape::Cuboid;
use specs::prelude::ParallelIterator;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Human {
    size: f32,
    objective: Vector2<f32>,
}

impl Human {
    fn calc_acceleration(
        &self,
        //position: &Position,
        kin: &Kinematics,
        //others: &[(&Position, &Human)],
    ) -> Vector2<f32> {
        let force: Vector2<f32> = -0.2 * kin.velocity;
        //
        // +force += Vector2::unit_y() * -200.;
        return force;
        /*
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
        */
    }
}

pub struct HumanUpdate;

impl<'a> System<'a> for HumanUpdate {
    type SystemData = (
        Read<'a, DeltaTime>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Human>,
    );

    fn run(&mut self, (delta, pos, mut kinematics, humans): Self::SystemData) {
        let _delta = delta.0;

        let _xx: Vec<(&Position, &Human)> = (&pos, &humans).join().collect();

        (&pos, &mut kinematics, &humans)
            .par_join()
            .for_each(|(p, k, h)| {
                if (h.objective - p.0).magnitude2() < 1. {
                    k.velocity = [0.0, 0.0].into();
                    return;
                }

                let acc = h.calc_acceleration(&k);
                k.acceleration += acc;
            })
    }
}

pub fn setup(world: &mut World) {
    const SCALE: f32 = 1000.;

    for _ in 0..10 {
        let size = 10.;

        let x: f32 = rand::random::<f32>() * SCALE;
        let y: f32 = rand::random::<f32>() * SCALE * 0.4;

        let eb = world
            .create_entity()
            .with(MeshRenderComponent::from((
                RectRender {
                    width: size * 2.,
                    height: size * 2.,
                    ..Default::default()
                },
                RectRender {
                    width: 200.,
                    height: 200.,
                    filled: false,
                    ..Default::default()
                },
            )))
            .with(Position([x, y].into()))
            .with(Kinematics::zero())
            .with(Human {
                size,
                objective: [SCALE * 5. - x, y].into(),
            })
            .with(Movable);

        let e = eb.build();
        //let shape = Ball::new(size);
        let shape = Cuboid::new([size, size].into());

        add_shape(world, e, [x, y].into(), shape);
    }
}
