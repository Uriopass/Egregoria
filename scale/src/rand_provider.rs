use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Float, Standard, StandardNormal};

pub struct RandProvider {
    pub rng: rand::rngs::SmallRng,
}

impl RandProvider {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
        }
    }

    pub fn random<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        self.rng.gen()
    }

    pub fn rand_normal<T: Float>(&mut self, mean: T, std: T) -> T
    where
        StandardNormal: Distribution<T>,
    {
        rand_distr::Normal::new(mean, std)
            .expect("Invalid normal distribution")
            .sample(&mut self.rng)
    }

    /// Gives number in range [min; max[
    pub fn rand_range(&mut self, min: i64, max: i64) -> i64 {
        rand_distr::Uniform::new(min, max).sample(&mut self.rng)
    }
}
