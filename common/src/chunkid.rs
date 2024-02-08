use geom::{Vec2, AABB};
use serde::{Deserialize, Serialize};

/// ChunkID is a tuple of two i16s, the first is the chunk's x coordinate, the second is the chunk's y coordinate.
/// It is parametrized by the LEVEL
/// Chunk size = 2^LEVEL * 16 (in meters)
/// ChunkID<0> can be useful for cars, ChunkID<1> for trees, ChunkID<4> for heightmap...
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChunkID<const LEVEL: u16>(pub i16, pub i16);

/// Minimum chunk size, in meters
const CHUNK_ID_BASE_SIZE: u32 = 16;
#[allow(non_camel_case_types)]
mod chunk_types {
    use super::ChunkID;
    pub type ChunkID_16 = ChunkID<0>;
    pub type ChunkID_32 = ChunkID<1>;
    pub type ChunkID_64 = ChunkID<2>;
    pub type ChunkID_128 = ChunkID<3>;
    pub type ChunkID_256 = ChunkID<4>;
    pub type ChunkID_512 = ChunkID<5>;
    pub type ChunkID_1024 = ChunkID<6>;
    pub type ChunkID_2048 = ChunkID<7>;
}
pub use chunk_types::*;

impl<const LEVEL: u16> ChunkID<LEVEL> {
    /// The size of a chunk in meters. Smallest at Level=0
    /// Doubles for each level
    pub const SIZE: u32 = CHUNK_ID_BASE_SIZE << LEVEL;
    pub const SIZE_F32: f32 = Self::SIZE as f32;

    pub fn new(p: Vec2) -> Self {
        Self(
            (p.x / Self::SIZE_F32).floor() as i16,
            (p.y / Self::SIZE_F32).floor() as i16,
        )
    }

    pub fn new_i16(x: i16, y: i16) -> Self {
        Self(x, y)
    }

    pub fn size(&self) -> f32 {
        Self::SIZE_F32
    }

    pub fn corner(self) -> Vec2 {
        Vec2::new(
            self.0 as f32 * Self::SIZE_F32,
            self.1 as f32 * Self::SIZE_F32,
        )
    }

    pub fn center(self) -> Vec2 {
        self.corner() + Vec2::splat(Self::SIZE_F32 / 2.0)
    }

    pub fn bbox(self) -> AABB {
        let ll = self.corner();
        AABB::new_ll_size(ll, Vec2::splat(Self::SIZE_F32))
    }

    pub fn convert_up<const NEW_LEVEL: u16>(self) -> ChunkID<NEW_LEVEL> {
        if NEW_LEVEL >= LEVEL {
            let scale = NEW_LEVEL - LEVEL;

            ChunkID(self.0 >> scale, self.1 >> scale)
        } else {
            debug_assert!(
                false,
                "Cannot convert to a lower level as that gives more than one chunkid per chunkid"
            );
            let scale = LEVEL - NEW_LEVEL;

            ChunkID(self.0 << scale, self.1 << scale)
        }
    }

    pub fn convert<const NEW_LEVEL: u16>(self) -> impl Iterator<Item = ChunkID<NEW_LEVEL>> {
        let ll: ChunkID<NEW_LEVEL>;
        let ur: ChunkID<NEW_LEVEL>;

        if NEW_LEVEL >= LEVEL {
            let scale = NEW_LEVEL - LEVEL;

            ll = ChunkID(self.0 >> scale, self.1 >> scale);
            ur = ChunkID(ll.0 + 1, ll.1 + 1);
        } else {
            let scale = LEVEL - NEW_LEVEL;

            ll = ChunkID(self.0 << scale, self.1 << scale);
            ur = ChunkID((self.0 + 1) << scale, (self.1 + 1) << scale);
        }

        (ll.1..ur.1).flat_map(move |y| (ll.0..ur.0).map(move |x| ChunkID(x, y)))
    }
}

#[cfg(test)]
mod tests {
    use crate::ChunkID;
    use geom::vec2;

    #[test]
    fn test_utils() {
        let c = ChunkID::<0>(-1, 1);

        assert_eq!(c.corner(), vec2(-16.0, 16.0));
        assert_eq!(c.center(), vec2(-8.0, 24.0));
        assert_eq!(
            c.bbox(),
            geom::AABB::new_ll_ur(vec2(-16.0, 16.0), vec2(0.0, 32.0))
        );
        assert_eq!(c.size(), 16.0);

        let c = ChunkID::<1>(-1, 1);

        assert_eq!(c.corner(), vec2(-32.0, 32.0));
        assert_eq!(c.center(), vec2(-16.0, 48.0));
        assert_eq!(
            c.bbox(),
            geom::AABB::new_ll_ur(vec2(-32.0, 32.0), vec2(0.0, 64.0))
        );
        assert_eq!(c.size(), 32.0);
    }

    #[test]
    fn test_convert_up() {
        let c = ChunkID::<0>(15, 12);

        assert_eq!(c.convert_up::<1>(), ChunkID::<1>(7, 6));
        assert_eq!(c.convert_up::<2>(), ChunkID::<2>(3, 3));
        assert_eq!(c.convert_up::<3>(), ChunkID::<3>(1, 1));
        assert_eq!(c.convert_up::<4>(), ChunkID::<4>(0, 0));
        assert_eq!(c.convert_up::<5>(), ChunkID::<5>(0, 0));
    }

    #[test]
    fn test_convert() {
        let c = ChunkID::<0>(15, 12);

        assert_eq!(
            c.convert::<1>().collect::<Vec<_>>(),
            vec![ChunkID::<1>(7, 6)]
        );
        assert_eq!(
            c.convert::<2>().collect::<Vec<_>>(),
            vec![ChunkID::<2>(3, 3)]
        );
        assert_eq!(
            c.convert::<3>().collect::<Vec<_>>(),
            vec![ChunkID::<3>(1, 1)]
        );
        assert_eq!(
            c.convert::<4>().collect::<Vec<_>>(),
            vec![ChunkID::<4>(0, 0)]
        );
        assert_eq!(
            c.convert::<5>().collect::<Vec<_>>(),
            vec![ChunkID::<5>(0, 0)]
        );

        let c = ChunkID::<4>(-1, 0);

        assert_eq!(
            c.convert::<3>().collect::<Vec<_>>(),
            vec![
                ChunkID::<3>(-2, 0),
                ChunkID::<3>(-1, 0),
                ChunkID::<3>(-2, 1),
                ChunkID::<3>(-1, 1)
            ]
        );

        let c = ChunkID::<4>(-3, 3);

        assert_eq!(
            c.convert::<3>().collect::<Vec<_>>(),
            vec![
                ChunkID::<3>(-6, 6),
                ChunkID::<3>(-5, 6),
                ChunkID::<3>(-6, 7),
                ChunkID::<3>(-5, 7)
            ]
        );

        assert_eq!(
            c.convert::<2>().collect::<Vec<_>>(),
            vec![
                ChunkID::<2>(-12, 12),
                ChunkID::<2>(-11, 12),
                ChunkID::<2>(-10, 12),
                ChunkID::<2>(-9, 12),
                //
                ChunkID::<2>(-12, 13),
                ChunkID::<2>(-11, 13),
                ChunkID::<2>(-10, 13),
                ChunkID::<2>(-9, 13),
                //
                ChunkID::<2>(-12, 14),
                ChunkID::<2>(-11, 14),
                ChunkID::<2>(-10, 14),
                ChunkID::<2>(-9, 14),
                //
                ChunkID::<2>(-12, 15),
                ChunkID::<2>(-11, 15),
                ChunkID::<2>(-10, 15),
                ChunkID::<2>(-9, 15)
            ]
        );
    }
}
