use crate::cars::car::CarComponent;
use crate::cars::car_graph::RoadGraph;
use crate::engine::components::{Kinematics, Position};
use specs::shred::PanicHandler;
use specs::{Join, Read, ReadStorage, System, World, WriteStorage};

#[derive(Default)]
pub struct CarDecision;

impl<'a> System<'a> for CarDecision {
    type SystemData = (
        Read<'a, RoadGraph, PanicHandler>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Kinematics>,
        ReadStorage<'a, CarComponent>,
    );

    fn run(&mut self, (_road_graph, positions, mut kinematics, cars): Self::SystemData) {
        for (pos, kin, car) in (&positions, &mut kinematics, &cars).join() {
            let (acc, ang_acc) = car.calc_decision(pos.0);
            kin.acceleration += acc;
        }
    }
}
