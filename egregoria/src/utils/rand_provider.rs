use rand::{Error, Rng, RngCore, SeedableRng};
use rand_distr::{Distribution, Float, Standard, StandardNormal};

pub struct RandProvider {
    rng: rand::rngs::SmallRng,
}

impl RngCore for RandProvider {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.rng.try_fill_bytes(dest)
    }
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
