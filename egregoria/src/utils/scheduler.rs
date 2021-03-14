use crate::{Egregoria, ParCommandBuffer};
use common::History;
use legion::systems::ParallelRunnable;
use ordered_float::OrderedFloat;
use std::time::Instant;

#[derive(Default)]
pub struct SeqSchedule {
    systems: Vec<(Box<dyn ParallelRunnable>, History)>,
}

impl SeqSchedule {
    pub fn add_system(&mut self, s: Box<dyn ParallelRunnable>) -> &mut Self {
        self.systems.push((s, History::new(100)));
        self
    }

    pub fn execute(&mut self, goria: &mut Egregoria) {
        for (sys, h) in &mut self.systems {
            let world = &mut goria.world;
            let res = &mut goria.resources;
            let start = Instant::now();

            sys.prepare(world);
            sys.run(world, res);

            if let Some(cb) = sys.command_buffer_mut(world.id()) {
                cb.flush(world, res);
            }
            ParCommandBuffer::apply(goria);

            let elapsed = start.elapsed();

            h.add_value(elapsed.as_secs_f32());
        }
    }

    pub fn times(&self) -> Vec<(String, f32)> {
        let mut times = self
            .systems
            .iter()
            .map(|(s, h)| (format!("{}", s.name().unwrap()), h.avg()))
            .collect::<Vec<_>>();
        times.sort_unstable_by_key(|(_, t)| OrderedFloat(-*t));
        times
    }
}
