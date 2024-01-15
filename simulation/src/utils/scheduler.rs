use crate::world::{CompanyEnt, HumanEnt, TrainEnt, VehicleEnt, WagonEnt};
use crate::{FreightStationEnt, ParCommandBuffer, Simulation};
use common::history::History;
use ordered_float::OrderedFloat;
use std::time::Instant;

pub trait RunnableSystem {
    fn run(&self, sim: &mut Simulation);
    fn name(&self) -> &'static str;
}

pub struct RunnableFn<F: Fn(&mut Simulation)> {
    pub f: F,
    pub name: &'static str,
}

impl<F: Fn(&mut Simulation)> RunnableSystem for RunnableFn<F> {
    fn run(&self, sim: &mut Simulation) {
        (self.f)(sim)
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Default)]
pub struct SeqSchedule {
    systems: Vec<(Box<dyn RunnableSystem>, History)>,
}

impl SeqSchedule {
    pub fn add_system(&mut self, s: Box<dyn RunnableSystem>) -> &mut Self {
        self.systems.push((s, History::new(100)));
        self
    }

    #[inline(never)]
    pub fn execute(&mut self, sim: &mut Simulation) {
        profiling::scope!("scheduler::execute");
        for (sys, h) in &mut self.systems {
            let start = Instant::now();

            sys.run(sim);

            ParCommandBuffer::<VehicleEnt>::apply(sim);
            ParCommandBuffer::<HumanEnt>::apply(sim);
            ParCommandBuffer::<TrainEnt>::apply(sim);
            ParCommandBuffer::<WagonEnt>::apply(sim);
            ParCommandBuffer::<FreightStationEnt>::apply(sim);
            ParCommandBuffer::<CompanyEnt>::apply(sim);

            let elapsed = start.elapsed();

            h.add_value(1000.0 * elapsed.as_secs_f32());
        }
    }

    pub fn times(&self) -> Vec<(String, f32)> {
        let mut times = self
            .systems
            .iter()
            .map(|(s, h)| (s.name().to_string(), h.avg()))
            .collect::<Vec<_>>();
        times.sort_unstable_by_key(|(_, t)| OrderedFloat(-*t));
        times
    }
}
