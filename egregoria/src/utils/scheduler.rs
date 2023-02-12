use crate::{Egregoria, ParCommandBuffer};
use common::History;
use ordered_float::OrderedFloat;
use std::time::Instant;

pub trait RunnableSystem {
    fn run(&self, goria: &mut Egregoria);
    fn name(&self) -> &'static str;
}

pub struct RunnableFn<F: Fn(&mut Egregoria)> {
    pub f: F,
    pub name: &'static str,
}

impl<F: Fn(&mut Egregoria)> RunnableSystem for RunnableFn<F> {
    fn run(&self, goria: &mut Egregoria) {
        (self.f)(goria)
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

    #[profiling::function]
    #[inline(never)]
    pub fn execute(&mut self, goria: &mut Egregoria) {
        for (sys, h) in &mut self.systems {
            let start = Instant::now();

            sys.run(goria);

            ParCommandBuffer::apply(goria);

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
