use core::num::Wrapping as w;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RandProvider {
    x: w<u32>,
    y: w<u32>,
    z: w<u32>,
    w: w<u32>,
}

impl RandProvider {
    pub fn new(mut seed: u64) -> Self {
        let tmp = splitmix64(&mut seed);
        let tmp2 = splitmix64(&mut seed);

        Self::from_seed([
            tmp as u32,
            (tmp >> 32) as u32,
            tmp2 as u32,
            (tmp2 >> 32) as u32,
        ])
    }

    pub fn next_f32(&mut self) -> f32 {
        f32::from_bits(0x3f800000 | (0x7fffff & self.next_u32())) - 1.0
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
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
    pub fn next_u64(&mut self) -> u64 {
        // Use LE; we explicitly generate one value before the next.
        let x = u64::from(self.next_u32());
        let y = u64::from(self.next_u32());
        (y << 32) | x
    }

    #[inline]
    #[allow(clippy::indexing_slicing)]
    /// Implement `fill_bytes` via `next_u64` and `next_u32`, little-endian order.
    ///
    /// The fastest way to fill a slice is usually to work as long as possible with
    /// integers. That is why this method mostly uses `next_u64`, and only when
    /// there are 4 or less bytes remaining at the end of the slice it uses
    /// `next_u32` once.
    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut left = dest;
        while left.len() >= 8 {
            let (l, r) = { left }.split_at_mut(8);
            left = r;
            let chunk: [u8; 8] = self.next_u64().to_le_bytes();
            l.copy_from_slice(&chunk);
        }
        let n = left.len();
        if n > 4 {
            let chunk: [u8; 8] = self.next_u64().to_le_bytes();
            left.copy_from_slice(&chunk[..n]);
        } else if n > 0 {
            let chunk: [u8; 4] = self.next_u32().to_le_bytes();
            left.copy_from_slice(&chunk[..n]);
        }
    }

    pub fn from_seed(mut seed_u32: [u32; 4]) -> Self {
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
}

// used only for the initial seed
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut result = *state;
    result = (result ^ (result >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    result = (result ^ (result >> 27)).wrapping_mul(0x94D049BB133111EB);
    result ^ (result >> 31)
}
