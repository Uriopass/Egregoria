use crate::physics::{Kinematics, Transform};
use cgmath::num_traits::zero;
use cgmath::InnerSpace;
use cgmath::Vector2;
use specs::{Component, Join, ReadStorage, System, VecStorage, WriteStorage};

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
            .join()
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
