use crate::interaction::{Movable, Selectable};
use crate::physics::add_shape;
use crate::physics::physics_components::{Kinematics, Transform};
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use cgmath::num_traits::zero;
use cgmath::{InnerSpace, Vector2};
use ncollide2d::shape::Ball;
use specs::prelude::ParallelIterator;
use specs::{
    Builder, Component, Join, ParJoin, ReadStorage, System, VecStorage, World, WorldExt,
    WriteStorage,
};

#[derive(Component)]
#[storage(VecStorage)]
pub struct Human {
    objective: Vector2<f32>,
}

impl Human {
    fn calc_acceleration(
        &self,
        //position: &transform,
        _kin: &Kinematics,
        //others: &[(&transform, &Human)],
    ) -> Vector2<f32> {
        zero()
    }
}

pub struct HumanUpdate;

impl<'a> System<'a> for HumanUpdate {
    type SystemData = (
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Human>,
    );

    fn run(&mut self, (transforms, mut kinematics, humans): Self::SystemData) {
        let _xx: Vec<(&Transform, &Human)> = (&transforms, &humans).join().collect();

        (&transforms, &mut kinematics, &humans)
            .par_join()
            .for_each(|(t, k, h)| {
                if (h.objective - t.position()).magnitude2() < 1.0 {
                    k.velocity = [0.0, 0.0].into();
                    return;
                }

                let acc = h.calc_acceleration(&k);
                k.acceleration += acc;
            })
    }
}

pub fn setup(world: &mut World) {
    const SCALE: f32 = 100.0;

    for _ in 0..1 {
        let size = 1.0;

        let x: f32 = rand::random::<f32>() * SCALE;
        let y: f32 = rand::random::<f32>() * SCALE;

        let eb = world
            .create_entity()
            .with(MeshRender::simple(
                CircleRender {
                    radius: size,
                    ..Default::default()
                },
                2,
            ))
            .with(Transform::new((x, y)))
            .with(Kinematics::from_mass(70.0))
            .with(Human {
                objective: [SCALE * 5.0 - x, y].into(),
            })
            .with(Selectable)
            .with(Movable);

        let e = eb.build();
        //let shape = Ball::new(size);
        let shape = Ball::new(size);

        add_shape(world, e, shape);
    }
}
