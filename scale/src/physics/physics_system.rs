use crate::PhysicsWorld;

use nalgebra as na;

use crate::engine_interaction::{DeltaTime, KeyCode, KeyboardInfo};
use crate::physics::physics_components::{Collider, Drag, Kinematics, Transform};
use cgmath::{InnerSpace, Vector2, Zero};
use nalgebra::Isometry2;
use specs::{Join, Read, ReadStorage, Write, WriteStorage};

pub struct KinematicsApply;

pub struct PhysicsUpdate {
    dynamic_collisions_enabled: bool,
}

impl Default for PhysicsUpdate {
    fn default() -> Self {
        PhysicsUpdate {
            dynamic_collisions_enabled: false,
        }
    }
}

const C_R: f32 = 0.2; // 0 for inelastic, 1 for elastic
impl<'a> specs::System<'a> for PhysicsUpdate {
    type SystemData = (
        Read<'a, KeyboardInfo>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        Write<'a, PhysicsWorld, specs::shred::PanicHandler>,
    );

    fn run(&mut self, (kb, mut transforms, mut kinematics, mut coworld): Self::SystemData) {
        coworld.update();

        if kb.just_pressed.contains(&KeyCode::P) {
            self.dynamic_collisions_enabled = !self.dynamic_collisions_enabled;
        }
        for (h1, h2, _alg, manifold) in coworld.contact_pairs(true) {
            let ent_1 = coworld.collision_object(h1).unwrap().data();
            let ent_2 = coworld.collision_object(h2).unwrap().data();

            let contact = manifold.deepest_contact().unwrap().contact;

            let normal: Vector2<f32> =
                Vector2::<f32>::new(contact.normal.x, contact.normal.y).normalize();

            let direction = normal * contact.depth;

            let is_dynamic_1 = kinematics.get(*ent_1).is_some();
            let is_dynamic_2 = kinematics.get(*ent_2).is_some();

            if is_dynamic_1 && is_dynamic_2 {
                if !self.dynamic_collisions_enabled {
                    continue;
                }
                let m_1 = kinematics.get(*ent_1).unwrap().mass;
                let m_2 = kinematics.get(*ent_2).unwrap().mass;

                // elastic collision
                let v_1 = kinematics.get(*ent_1).unwrap().velocity;
                let v_2 = kinematics.get(*ent_2).unwrap().velocity;

                let r_1 = (1.0 + C_R) * m_2 / (m_1 + m_2);
                let r_2 = (1.0 + C_R) * m_1 / (m_1 + m_2);

                let v_diff: Vector2<f32> = v_1 - v_2;
                let factor = normal.dot(v_diff);

                kinematics.get_mut(*ent_1).unwrap().velocity -= r_1 * factor * normal;
                kinematics.get_mut(*ent_2).unwrap().velocity += r_2 * factor * normal;

                let f_1 = m_2 / (m_1 + m_2);
                let f_2 = 1.0 - f_1;
                transforms
                    .get_mut(*ent_1)
                    .unwrap()
                    .translate(-direction * f_1);
                transforms
                    .get_mut(*ent_2)
                    .unwrap()
                    .translate(direction * f_2);
                continue;
            }
            if is_dynamic_1 {
                let pos_1 = transforms.get_mut(*ent_1).unwrap();
                pos_1.translate(-direction);

                let k_1 = kinematics.get_mut(*ent_1).unwrap();
                let projected = k_1.velocity.project_on(normal) * -2.0;
                k_1.velocity += projected;
                continue;
            }

            if is_dynamic_2 {
                let pos_2 = transforms.get_mut(*ent_2).unwrap();
                pos_2.translate(direction);

                let k_2 = kinematics.get_mut(*ent_2).unwrap();
                let projected = k_2.velocity.project_on(-normal) * -2.0;
                k_2.velocity += projected;
            }
        }
    }
}

const RHO: f32 = 1.2;

impl<'a> specs::System<'a> for KinematicsApply {
    type SystemData = (
        WriteStorage<'a, Collider>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, Drag>,
        Write<'a, PhysicsWorld, specs::shred::PanicHandler>,
        Read<'a, DeltaTime>,
    );

    fn run(
        &mut self,
        (mut collider, mut transforms, mut kinematics, drag, mut ncollide_world, delta): Self::SystemData,
    ) {
        let delta = delta.0;

        for (kin, drag) in (&mut kinematics, &drag).join() {
            let force = kin.velocity.magnitude2() * drag.coeff * (RHO / 2.0) / kin.mass;
            if force == 0.0 {
                continue;
            }

            kin.acceleration += kin.velocity.normalize_to(-force);
        }

        for (transform, kin) in (&mut transforms, &mut kinematics).join() {
            kin.velocity += kin.acceleration * delta;
            transform.translate(kin.velocity * delta);
            kin.acceleration.set_zero();
        }

        for (transform, collider) in (&mut transforms, &mut collider).join() {
            let collision_obj = ncollide_world
                .get_mut(collider.0)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
            let p = transform.position();
            let iso = Isometry2::from_parts(
                na::Translation2::new(p.x, p.y),
                na::UnitComplex::new_unchecked(na::Complex::new(
                    transform.get_cos(),
                    transform.get_sin(),
                )),
            );

            collision_obj.set_position(iso);
        }
    }
}
