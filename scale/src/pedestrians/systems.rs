use crate::engine_interaction::TimeInfo;
use crate::pedestrians::PedestrianComponent;
use crate::physics::Transform;
use specs::prelude::*;

#[derive(Default)]
pub struct PedestrianDecision;

#[derive(SystemData)]
pub struct PedestrianDecisionData<'a> {
    time: Read<'a, TimeInfo>,
    transforms: ReadStorage<'a, Transform>,
    pedestrians: WriteStorage<'a, PedestrianComponent>,
}

impl<'a> System<'a> for PedestrianDecision {
    type SystemData = PedestrianDecisionData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let delta: f32 = data.time.delta;
    }
}
