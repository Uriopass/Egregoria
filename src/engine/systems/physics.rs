use crate::engine::components::{Collider, Kinematics, Position};
use crate::engine::resources::DeltaTime;
use crate::PhysicsWorld;

use nalgebra as na;

use cgmath::{InnerSpace, Vector2, Zero};
use nalgebra::Isometry2;
use ncollide2d::bounding_volume::AABB;
use ncollide2d::pipeline::{
    CollisionGroups, CollisionObject, CollisionObjectSet, CollisionObjectSlab,
    CollisionObjectSlabHandle, InterferencesWithAABB,
};
use specs::{Entity, Join, Read, Write, WriteStorage};

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

            let direction = normal * (contact.depth + 0.01);

            let is_dynamic_1 = kinematics.get(*ent_1).is_some();
            let is_dynamic_2 = kinematics.get(*ent_2).is_some();

            let m_1 = 1.;
            let m_2 = 1.;

            if is_dynamic_1 && is_dynamic_2 {
                // elastic collision
                let pos_1 = positions.get(*ent_1).unwrap().0;
                let pos_2 = positions.get(*ent_2).unwrap().0;

                let aaaaa = AABB::new(
                    na::Point2::new(pos_1.x - 100., pos_1.y - 100.),
                    na::Point2::new(pos_1.x + 100., pos_1.y + 100.),
                );
                let cggg = Default::default();
                let test = coworld.interferences_with_aabb(&aaaaa, &cggg);

                let objs: Vec<(CollisionObjectSlabHandle, &CollisionObject<f32, Entity>)> =
                    test.collect();

                println!("Collision! {} objects around ", objs.len());

                let v_1 = kinematics.get(*ent_1).unwrap().velocity;
                let v_2 = kinematics.get(*ent_2).unwrap().velocity;

                let r_1 = 2. * m_2 / (m_1 + m_2);
                let r_2 = 2. * m_1 / (m_1 + m_2);

                let v_diff: Vector2<f32> = v_1 - v_2;
                let pos_diff: Vector2<f32> = pos_1 - pos_2;
                let factor = pos_diff.dot(v_diff) / pos_diff.magnitude2();

                kinematics.get_mut(*ent_1).unwrap().velocity -= r_1 * factor * pos_diff;
                kinematics.get_mut(*ent_2).unwrap().velocity += r_2 * factor * pos_diff;

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

impl<'a> specs::System<'a> for KinematicsApply {
    type SystemData = (
        WriteStorage<'a, Collider>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        // Gotta use the panic handler here 'cause there is no default
        // we can provide for CollisionWorld, I guess.
        Write<'a, PhysicsWorld, specs::shred::PanicHandler>,
        Read<'a, DeltaTime>,
    );

    fn run(
        &mut self,
        (mut collider, mut position, mut kinematics, mut ncollide_world, delta): Self::SystemData,
    ) {
        let delta = delta.0;

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
