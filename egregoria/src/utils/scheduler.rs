use crate::utils::frame_log::FrameLog;
use legion::systems::ParallelRunnable;
use legion::{Resources, World};
use std::time::Instant;

#[derive(Default)]
pub struct SeqSchedule {
    systems: Vec<Box<dyn ParallelRunnable>>,
}

impl SeqSchedule {
    pub fn add_system(&mut self, s: impl ParallelRunnable + 'static) -> &mut Self {
        self.systems.push(Box::new(s));
        self
    }

    pub fn execute(&mut self, world: &mut World, res: &mut Resources) {
        for sys in &mut self.systems {
            let start = Instant::now();

            sys.prepare(world);
            sys.run(world, res);
            if let Some(cb) = sys.command_buffer_mut(world.id()) {
                cb.flush(world);
            }

            let elapsed = start.elapsed();

            let s = format!(
                "system {} took {:.2}ms",
                sys.name().unwrap(),
                elapsed.as_secs_f32() * 1000.0
            );
            res.get::<FrameLog>().unwrap().log_frame(s);
        }
    }
}
