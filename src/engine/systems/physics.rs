use crate::engine::components::{Collider, Position, Velocity};
use crate::engine::resources::DeltaTime;
use crate::PhysicsWorld;

use nalgebra as na;

use cgmath::{InnerSpace, Vector2, Zero};
use nalgebra::Isometry2;
use specs::{Join, Read, ReadStorage, Write, WriteStorage};

pub struct SpeedApply;
pub struct PhysicsUpdate;

impl<'a> specs::System<'a> for PhysicsUpdate {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        Write<'a, PhysicsWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (mut position, mut velocity, mut ncollide_world): Self::SystemData) {
        ncollide_world.update();

        for (h1, h2, _alg, manifold) in ncollide_world.contact_pairs(true) {
            let ent_1 = ncollide_world.collision_object(h1).unwrap().data();
            let ent_2 = ncollide_world.collision_object(h2).unwrap().data();

            let contact = manifold.deepest_contact().unwrap().contact;

            let normal: Vector2<f32> =
                Vector2::<f32>::new(contact.normal.x, contact.normal.y).normalize();

            let direction = normal * contact.depth;

            let has_velocity_1 = velocity.get(*ent_1).is_some();
            let has_velocity_2 = velocity.get(*ent_2).is_some();

            let m_1 = 1.;
            let m_2 = 1.;

            if has_velocity_1 && has_velocity_2 {
                // elastic collision
                let pos_1 = position.get(*ent_1).unwrap();
                let pos_2 = position.get(*ent_2).unwrap();

                let v_1 = velocity.get(*ent_1).unwrap();
                let v_2 = velocity.get(*ent_2).unwrap();

                let r_1 = 2. * m_2 / (m_1 + m_2);
                let r_2 = 2. * m_1 / (m_1 + m_2);

                let v_diff: Vector2<f32> = v_1.0 - v_2.0;
                let pos_diff: Vector2<f32> = pos_1.0 - pos_2.0;
                let factor = pos_diff.dot(v_diff) / pos_diff.magnitude2();

                velocity.get_mut(*ent_1).unwrap().0 -= r_1 * factor * pos_diff;
                velocity.get_mut(*ent_2).unwrap().0 += r_2 * factor * pos_diff;

                velocity.get_mut(*ent_1).unwrap().0 *= 0.99;
                velocity.get_mut(*ent_2).unwrap().0 *= 0.99;

                position.get_mut(*ent_1).unwrap().0 -= direction / 2.;
                position.get_mut(*ent_2).unwrap().0 += direction / 2.;
            } else if has_velocity_1 {
                let pos_1 = position.get_mut(*ent_1).unwrap();
                pos_1.0 -= direction;

                let v_1 = velocity.get_mut(*ent_1).unwrap();
                let projected = v_1.0.project_on(normal) * -2.;
                v_1.0 += projected;
            } else if has_velocity_2 {
                let pos_2 = position.get_mut(*ent_2).unwrap();
                pos_2.0 += direction;

                let v_2 = velocity.get_mut(*ent_2).unwrap();
                let projected = v_2.0.project_on(-normal) * -2.;
                v_2.0 += projected;
            }
        }
    }
}

impl<'a> specs::System<'a> for SpeedApply {
    type SystemData = (
        WriteStorage<'a, Collider>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
        // Gotta use the panic handler here 'cause there is no default
        // we can provide for CollisionWorld, I guess.
        Write<'a, PhysicsWorld, specs::shred::PanicHandler>,
        Read<'a, DeltaTime>,
    );

    fn run(
        &mut self,
        (mut collider, mut position, velocity, mut ncollide_world, delta): Self::SystemData,
    ) {
        let delta = delta.0;

        for (position, velocity) in (&mut position, &velocity).join() {
            position.0 += velocity.0 * delta;
        }

        for (collider, position, _velocity) in (&mut collider, &mut position, &velocity).join() {
            let collision_obj = ncollide_world
                .get_mut(collider.0)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
            collision_obj.set_position(Isometry2::new(
                na::Vector2::new(position.0.x, position.0.y),
                na::zero(),
            ));
        }
    }
}
