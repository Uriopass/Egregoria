use crate::{Egregoria, ParCommandBuffer};
use common::History;
use hecs::World;
use ordered_float::OrderedFloat;
use resources::Resources;
use std::time::Instant;

pub trait RunnableSystem {
    fn run(&mut self, world: &mut World, res: &mut Resources);
    fn name(&self) -> &'static str;
}

pub struct RunnableFn<F: FnMut(&mut World, &mut Resources)> {
    pub f: F,
    pub name: &'static str,
}

impl<F: FnMut(&mut World, &mut Resources)> RunnableSystem for RunnableFn<F> {
    fn run(&mut self, world: &mut World, res: &mut Resources) {
        (self.f)(world, res)
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
    pub fn execute(&mut self, goria: &mut Egregoria) {
        for (sys, h) in &mut self.systems {
            let world = &mut goria.world;
            let res = &mut goria.resources;
            let start = Instant::now();

            sys.run(world, res);

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
