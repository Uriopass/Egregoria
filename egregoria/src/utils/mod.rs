use crate::Egregoria;

pub mod par_command_buffer;
pub mod rand_provider;
pub mod scheduler;
pub mod time;

pub fn rand_world<T>(world: &Egregoria) -> T
where
    rand_distr::Standard: rand_distr::Distribution<T>,
{
    world.write::<crate::RandProvider>().random()
}
