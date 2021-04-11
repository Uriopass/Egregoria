use core::num::Wrapping as w;
use rand::distributions::Standard;
use rand::{Error, RngCore, SeedableRng};
use rand_distr::num_traits::Float;
use rand_distr::{Distribution, StandardNormal};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize)]
pub struct RandProvider {
    x: w<u32>,
    y: w<u32>,
    z: w<u32>,
    w: w<u32>,
}

impl RandProvider {
    pub fn new(seed: u64) -> Self {
        Self::seed_from_u64(seed)
    }

    pub fn random<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        Standard.sample(self)
    }

    pub fn rand_normal<T: Float>(&mut self, mean: T, std: T) -> T
    where
        StandardNormal: Distribution<T>,
    {
        rand_distr::Normal::new(mean, std)
            .expect("Invalid normal distribution")
            .sample(self)
    }
}

impl RngCore for RandProvider {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let x = self.x;
        let t = x ^ (x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let w_ = self.w;
        self.w = w_ ^ (w_ >> 19) ^ (t ^ (t >> 8));
        self.w.0
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        // Use LE; we explicitly generate one value before the next.
        let x = u64::from(self.next_u32());
        let y = u64::from(self.next_u32());
        (y << 32) | x
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[allow(clippy::unwrap_used)]
impl SeedableRng for RandProvider {
    type Seed = [u8; 16];

    fn from_seed(seed: Self::Seed) -> Self {
        let mut seed_u32 = [0u32; 4];
        for (out, chunk) in seed_u32.iter_mut().zip((&seed).chunks_exact(4)) {
            *out = u32::from_le_bytes(chunk.try_into().unwrap());
        }

        // Xorshift cannot be seeded with 0 and we cannot return an Error, but
        // also do not wish to panic (because a random seed can legitimately be
        // 0); our only option is therefore to use a preset value.
        if seed_u32.iter().all(|&x| x == 0) {
            seed_u32 = [0xBAD_5EED, 0xBAD_5EED, 0xBAD_5EED, 0xBAD_5EED];
        }

        Self {
            x: w(seed_u32[0]),
            y: w(seed_u32[1]),
            z: w(seed_u32[2]),
            w: w(seed_u32[3]),
        }
    }

    fn from_rng<R: RngCore>(mut rng: R) -> Result<Self, Error> {
        let mut b = [0u8; 16];
        loop {
            rng.try_fill_bytes(&mut b[..])?;
            if !b.iter().all(|&x| x == 0) {
                break;
            }
        }

        Ok(Self {
            x: w(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
            y: w(u32::from_le_bytes([b[4], b[5], b[6], b[7]])),
            z: w(u32::from_le_bytes([b[8], b[9], b[10], b[11]])),
            w: w(u32::from_le_bytes([b[12], b[13], b[14], b[15]])),
        })
    }
}

#[allow(clippy::indexing_slicing)]
/// Implement `fill_bytes` via `next_u64` and `next_u32`, little-endian order.
///
/// The fastest way to fill a slice is usually to work as long as possible with
/// integers. That is why this method mostly uses `next_u64`, and only when
/// there are 4 or less bytes remaining at the end of the slice it uses
/// `next_u32` once.
pub fn fill_bytes_via_next<R: RngCore + ?Sized>(rng: &mut R, dest: &mut [u8]) {
    let mut left = dest;
    while left.len() >= 8 {
        let (l, r) = { left }.split_at_mut(8);
        left = r;
        let chunk: [u8; 8] = rng.next_u64().to_le_bytes();
        l.copy_from_slice(&chunk);
    }
    let n = left.len();
    if n > 4 {
        let chunk: [u8; 8] = rng.next_u64().to_le_bytes();
        left.copy_from_slice(&chunk[..n]);
    } else if n > 0 {
        let chunk: [u8; 4] = rng.next_u32().to_le_bytes();
        left.copy_from_slice(&chunk[..n]);
    }
}
