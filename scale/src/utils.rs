use lazy_static::*;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Float, Standard, StandardNormal};
use std::sync::Mutex;

macro_rules! unwrap_or {
    ($e: expr, $t: tt) => {
        match $e {
            Some(x) => x,
            None => $t,
        }
    };
}

lazy_static! {
    pub static ref RAND_STATE: Mutex<rand::rngs::SmallRng> =
        Mutex::new(rand::rngs::SmallRng::seed_from_u64(123));
}

pub fn rand_det<T>() -> T
where
    Standard: Distribution<T>,
{
    RAND_STATE.lock().unwrap().gen()
}

pub fn rand_normal<T: Float>(mean: T, std: T) -> T
where
    StandardNormal: Distribution<T>,
{
    let l = RAND_STATE.lock();
    rand_distr::Normal::new(mean, std)
        .unwrap()
        .sample(&mut (*l.unwrap()))
}

/// Gives number in range [min; max[
pub fn rand_range(min: i64, max: i64) -> i64 {
    let l = RAND_STATE.lock();
    rand_distr::Uniform::new(min, max).sample(&mut (*l.unwrap()))
}

pub trait Choose<'a> {
    type Output;
    fn choose(&'a self) -> Self::Output;
}

impl<'a, T: 'a> Choose<'a> for Vec<T> {
    type Output = Option<&'a T>;

    fn choose(&'a self) -> Self::Output {
        if self.is_empty() {
            None
        } else {
            let l = self.len();
            let ix = (l as f32 * crate::utils::rand_det::<f32>()) as usize;
            Some(&self[ix])
        }
    }
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
