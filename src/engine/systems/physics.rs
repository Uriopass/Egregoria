use crate::engine::components::{Collider, Drag, Kinematics, Position};
use crate::engine::resources::DeltaTime;
use crate::PhysicsWorld;

use nalgebra as na;

use cgmath::{InnerSpace, Vector2, Zero};
use nalgebra::Isometry2;

use specs::{Join, Read, ReadStorage, Write, WriteStorage};

pub struct KinematicsApply;
pub struct PhysicsUpdate;

impl<'a> specs::System<'a> for PhysicsUpdate {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        Write<'a, PhysicsWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (mut positions, mut kinematics, mut coworld): Self::SystemData) {
        coworld.update();

        for (h1, h2, _alg, manifold) in coworld.contact_pairs(true) {
            let ent_1 = coworld.collision_object(h1).unwrap().data();
            let ent_2 = coworld.collision_object(h2).unwrap().data();

            let contact = manifold.deepest_contact().unwrap().contact;

            let normal: Vector2<f32> =
                Vector2::<f32>::new(contact.normal.x, contact.normal.y).normalize();

            let direction = normal * contact.depth;

            let is_dynamic_1 = kinematics.get(*ent_1).is_some();
            let is_dynamic_2 = kinematics.get(*ent_2).is_some();

            let m_1 = 1.;
            let m_2 = 1.;

            if is_dynamic_1 && is_dynamic_2 {
                // elastic collision
                let v_1 = kinematics.get(*ent_1).unwrap().velocity;
                let v_2 = kinematics.get(*ent_2).unwrap().velocity;

                let r_1 = 2. * m_2 / (m_1 + m_2);
                let r_2 = 2. * m_1 / (m_1 + m_2);

                let v_diff: Vector2<f32> = v_1 - v_2;
                let factor = normal.dot(v_diff);

                kinematics.get_mut(*ent_1).unwrap().velocity -= r_1 * factor * normal;
                kinematics.get_mut(*ent_2).unwrap().velocity += r_2 * factor * normal;

                positions.get_mut(*ent_1).unwrap().0 -= direction / 2.;
                positions.get_mut(*ent_2).unwrap().0 += direction / 2.;
            } else if is_dynamic_1 {
                let pos_1 = positions.get_mut(*ent_1).unwrap();
                pos_1.0 -= direction;

                let k_1 = kinematics.get_mut(*ent_1).unwrap();
                let projected = k_1.velocity.project_on(normal) * -2.;
                k_1.velocity += projected;
            } else if is_dynamic_2 {
                let pos_2 = positions.get_mut(*ent_2).unwrap();
                pos_2.0 += direction;

                let k_2 = kinematics.get_mut(*ent_2).unwrap();
                let projected = k_2.velocity.project_on(-normal) * -2.;
                k_2.velocity += projected;
            }
        }
    }
}

const DRAG_COEFF: f32 = 0.2;

impl<'a> specs::System<'a> for KinematicsApply {
    type SystemData = (
        WriteStorage<'a, Collider>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Drag>,
        // Gotta use the panic handler here 'cause there is no default
        // we can provide for CollisionWorld, I guess.
        Write<'a, PhysicsWorld, specs::shred::PanicHandler>,
        Read<'a, DeltaTime>,
    );

    fn run(
        &mut self,
        (mut collider, mut position, mut kinematics, drag, mut ncollide_world, delta): Self::SystemData,
    ) {
        let delta = delta.0;

        for (kin, drag) in (&mut kinematics, &drag).join() {
            kin.acceleration -= kin.velocity * drag.0;
        }

        for (position, kin) in (&mut position, &mut kinematics).join() {
            kin.velocity += kin.acceleration * delta;
            position.0 += kin.velocity * delta;
            kin.acceleration.set_zero();
        }

        for (position, collider) in (&mut position, &mut collider).join() {
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
