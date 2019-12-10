use crate::engine::components::{Position, Velocity};
use crate::engine::resources::DeltaTime;
use specs::{Join, Read, ReadStorage, System, Write, WriteStorage};

pub struct SpeedApply;

impl<'a> System<'a> for SpeedApply {
    type SystemData = (
        Read<'a, DeltaTime>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (delta, mut pos, vel): Self::SystemData) {
        let delta = delta.0;

        for (vel, pos) in (&vel, &mut pos).join() {
            pos.0 += vel.0 * delta;
        }
    }
}

pub struct CollisionSystem;

impl<'a> System<'a> for CollisionSystem {
    type SystemData = (Read<'a, DeltaTime>);

    fn run(&mut self, _update_time: Self::SystemData) {}
}
