use crate::engine::components::{CircleRender, Collider, Position, Velocity};
use crate::engine::resources::DeltaTime;
use crate::PhysicsWorld;
use ggez::graphics::Color;
use nalgebra as na;
use ncollide2d::pipeline::ContactEvent;

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
        WriteStorage<'a, CircleRender>,
    );

    fn run(
        &mut self,
        (mut collider, mut position, velocity, mut ncollide_world, delta, mut circle): Self::SystemData,
    ) {
        let delta = delta.0;
        for (collider, position, velocity) in (&mut collider, &mut position, &velocity).join() {
            position.0 += velocity.0 * delta;

            let collision_obj = ncollide_world
                .get_mut(collider.0)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
            collision_obj.set_position(Isometry2::new(
                na::Vector2::new(position.0.x, position.0.y),
                na::zero(),
            ));
        }

        ncollide_world.update();
        for ev in ncollide_world.contact_events() {
            println!("Contact event: {:?}", ev);
            match ev {
                ContactEvent::Started(h1, h2) => {
                    let ent = ncollide_world.collision_object(*h1).unwrap();
                    let e = circle.get_mut(*ent.data()).unwrap();
                    e.color = Color::new(1., 0., 0., 1.);

                    let ent = ncollide_world.collision_object(*h2).unwrap();
                    let e = circle.get_mut(*ent.data()).unwrap();
                    e.color = Color::new(1., 0., 0., 1.);
                }
                ContactEvent::Stopped(h1, h2) => {
                    let ent = ncollide_world.collision_object(*h1).unwrap();
                    let e = circle.get_mut(*ent.data()).unwrap();
                    e.color = Color::new(1., 1., 1., 1.);

                    let ent = ncollide_world.collision_object(*h2).unwrap();
                    let e = circle.get_mut(*ent.data()).unwrap();
                    e.color = Color::new(1., 1., 1., 1.);
                }
            }
        }
    }
}
