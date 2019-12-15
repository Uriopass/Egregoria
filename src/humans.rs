use cgmath::{InnerSpace, Vector2};

use specs::{
    Builder, Component, Join, ParJoin, Read, ReadStorage, System, VecStorage, World, WorldExt,
    WriteStorage,
};

use crate::add_shape;
use crate::engine::components::{
    CircleRender, Drag, Kinematics, LineRender, MeshRenderComponent, MeshRenderable, Movable,
    RectRender, Transform,
};
use crate::engine::resources::DeltaTime;

use cgmath::num_traits::zero;
use ncollide2d::shape::Ball;
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
        //position: &transform,
        _kin: &Kinematics,
        //others: &[(&transform, &Human)],
    ) -> Vector2<f32> {
        let force: Vector2<f32> = zero();
        //
        // +force += Vector2::unit_y() * -200.;
        return force;
        /*
        force += (self.objective -transform.0).normalize() * 20.;

        for (p, h) in others {
            let mut x: Vector2<f32> =transform.0 - p.0;
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
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Human>,
    );

    fn run(&mut self, (delta, transforms, mut kinematics, humans): Self::SystemData) {
        let _delta = delta.0;

        let _xx: Vec<(&Transform, &Human)> = (&transforms, &humans).join().collect();

        (&transforms, &mut kinematics, &humans)
            .par_join()
            .for_each(|(t, k, h)| {
                if (h.objective - t.get_position()).magnitude2() < 1. {
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
            .with(MeshRenderComponent::from(CircleRender {
                radius: size,
                ..Default::default()
            }))
            .with(Transform::new([x, y].into()))
            .with(Kinematics::zero())
            .with(Drag::default())
            .with(Human {
                size,
                objective: [SCALE * 5. - x, y].into(),
            })
            .with(Movable);

        let e = eb.build();
        //let shape = Ball::new(size);
        let shape = Ball::new(size);

        add_shape(world, e, [x, y].into(), shape);
    }
}
