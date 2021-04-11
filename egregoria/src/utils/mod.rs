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

pub trait Restrict {
    fn restrict(self, min: Self, max: Self) -> Self;
}

impl<T: PartialOrd> Restrict for T {
    fn restrict(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}
