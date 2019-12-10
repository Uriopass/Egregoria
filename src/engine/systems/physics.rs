use crate::engine::components::{CircleRender, Collider, Position, Velocity};
use crate::engine::resources::DeltaTime;
use crate::PhysicsWorld;

use nalgebra as na;

use cgmath::{Vector2, Zero};
use nalgebra::Isometry2;
use specs::{Join, Read, ReadStorage, Write, WriteStorage};

pub struct SpeedApply;

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

        ncollide_world.update();
        for (h1, h2, _alg, manifold) in ncollide_world.contact_pairs(true) {
            let ent_1 = ncollide_world.collision_object(h1).unwrap().data();
            let ent_2 = ncollide_world.collision_object(h2).unwrap().data();

            let contact = manifold.deepest_contact().unwrap().contact;

            let mut direction =
                Vector2::<f32>::new(contact.normal.x, contact.normal.y) * contact.depth;

            let has_velocity_1 = velocity.get(*ent_1).map_or(false, |x| !x.0.is_zero());
            let has_velocity_2 = velocity.get(*ent_2).map_or(false, |x| !x.0.is_zero());

            if has_velocity_1 && has_velocity_2 {
                direction /= 2.;
            }
            if has_velocity_1 {
                let pos_1 = position.get_mut(*ent_1).unwrap();
                (*pos_1).0 -= direction;
            }
            if has_velocity_2 {
                let pos_2 = position.get_mut(*ent_2).unwrap();
                (*pos_2).0 += direction;
            }
        }
    }
}
