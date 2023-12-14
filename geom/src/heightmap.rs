use crate::{vec2, Vec2, AABB};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};

pub type HeightmapChunkID = (u16, u16);

#[derive(Clone)]
pub struct HeightmapChunk<const RESOLUTION: usize, const SIZE: u32> {
    heights: [[f32; RESOLUTION]; RESOLUTION], // TODO: change to RESOLUTION * RESOLUTION when generic_const_exprs is stabilized
}

impl<const RESOLUTION: usize, const SIZE: u32> Default for HeightmapChunk<RESOLUTION, SIZE> {
    fn default() -> Self {
        Self {
            heights: [[0.0; RESOLUTION]; RESOLUTION],
        }
    }
}

impl<const RESOLUTION: usize, const SIZE: u32> HeightmapChunk<RESOLUTION, SIZE> {
    pub fn new(heights: [[f32; RESOLUTION]; RESOLUTION]) -> Self {
        Self { heights }
    }

    pub fn rect(id: HeightmapChunkID) -> AABB {
        let ll = vec2(id.0 as f32 * SIZE as f32, id.1 as f32 * SIZE as f32);
        let ur = ll + vec2(SIZE as f32, SIZE as f32);
        AABB::new(ll, ur)
    }

    pub fn id(v: Vec2) -> HeightmapChunkID {
        if v.x < 0.0 || v.y < 0.0 {
            return (0, 0);
        }
        ((v.x / SIZE as f32) as u16, (v.y / SIZE as f32) as u16)
    }

    /// assume p is in chunk-space and in-bounds
    pub fn height_unchecked(&self, p: Vec2) -> f32 {
        let v = p / SIZE as f32;
        let v = v * RESOLUTION as f32;
        self.heights[v.y as usize][v.x as usize]
    }

    pub fn heights(&self) -> &[[f32; RESOLUTION]; RESOLUTION] {
        &self.heights
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Heightmap<const RESOLUTION: usize, const SIZE: u32> {
    chunks: Vec<HeightmapChunk<RESOLUTION, SIZE>>, // chunks is an array of length w * h, indexed with (x + y * w)
    pub w: u16,
    pub h: u16,
}

impl<const RESOLUTION: usize, const SIZE: u32> Heightmap<RESOLUTION, SIZE> {
    pub const CELL_SIZE: f32 = SIZE as f32 / RESOLUTION as f32;
    pub const RESOLUTION: usize = RESOLUTION;
    pub const SIZE: u32 = SIZE;

    pub fn new(w: u16, h: u16) -> Self {
        Self {
            chunks: vec![HeightmapChunk::default(); (w * h) as usize],
            w,
            h,
        }
    }

    pub fn bounds(&self) -> AABB {
        AABB::new(
            vec2(0.0, 0.0),
            vec2(self.w as f32 * SIZE as f32, self.h as f32 * SIZE as f32),
        )
    }

    fn check_valid(&self, id: HeightmapChunkID) -> bool {
        id.0 < self.w && id.1 < self.h
    }

    pub fn set_chunk(&mut self, id: HeightmapChunkID, chunk: HeightmapChunk<RESOLUTION, SIZE>) {
        if !self.check_valid(id) {
            return;
        }
        self.chunks[(id.0 + id.1 * self.w) as usize] = chunk;
    }

    pub fn get_chunk(&self, id: HeightmapChunkID) -> Option<&HeightmapChunk<RESOLUTION, SIZE>> {
        if !self.check_valid(id) {
            return None;
        }
        unsafe { Some(self.chunks.get_unchecked((id.0 + id.1 * self.w) as usize)) }
    }

    pub fn chunks(
        &self,
    ) -> impl Iterator<Item = (HeightmapChunkID, &HeightmapChunk<RESOLUTION, SIZE>)> + '_ {
        self.chunks
            .iter()
            .enumerate()
            .map(move |(i, chunk)| ((i as u16 % self.w, i as u16 / self.w), chunk))
    }

    pub fn height_nearest(&self, p: Vec2) -> Option<f32> {
        let cell = HeightmapChunk::<RESOLUTION, SIZE>::id(p);

        self.get_chunk(cell).and_then(|chunk| {
            let v = p / SIZE as f32 - vec2(cell.0 as f32, cell.1 as f32);
            let v = v * RESOLUTION as f32;
            chunk
                .heights
                .get(v.y as usize)
                .and_then(|x| x.get(v.x as usize))
                .copied()
        })
    }

    /// Returns height at any point using bilinear interpolation
    pub fn height(&self, p: Vec2) -> Option<f32> {
        let exact = self.height_nearest(p);
        if let (Some(ll), Some(lr), Some(ul), Some(ur)) = (
            exact,
            self.height_nearest(p + Vec2::x(Self::CELL_SIZE)),
            self.height_nearest(p + Vec2::y(Self::CELL_SIZE)),
            self.height_nearest(p + vec2(Self::CELL_SIZE, Self::CELL_SIZE)),
        ) {
            let x = (p.x % Self::CELL_SIZE) / Self::CELL_SIZE;
            let y = (p.y % Self::CELL_SIZE) / Self::CELL_SIZE;

            let h01 = ll + x * (lr - ll);
            let h23 = ul + x * (ur - ul);

            return Some(h01 + y * (h23 - h01));
        }
        exact
    }
}

impl<const RESOLUTION: usize, const SIZE: u32> Serialize for HeightmapChunk<RESOLUTION, SIZE> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(RESOLUTION * RESOLUTION))?;
        for row in &self.heights {
            for height in row {
                seq.serialize_element(height)?;
            }
        }
        seq.end()
    }
}

impl<'de, const RESOLUTION: usize, const SIZE: u32> Deserialize<'de>
    for HeightmapChunk<RESOLUTION, SIZE>
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let heights = deserializer.deserialize_seq(HeightmapChunkVisitor::<RESOLUTION>)?;
        Ok(Self { heights })
    }
}

struct HeightmapChunkVisitor<const RESOLUTION: usize>;

impl<'de, const RESOLUTION: usize> serde::de::Visitor<'de> for HeightmapChunkVisitor<RESOLUTION> {
    type Value = [[f32; RESOLUTION]; RESOLUTION];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of floats")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let len = seq.size_hint().unwrap_or(RESOLUTION * RESOLUTION);
        if len != RESOLUTION * RESOLUTION {
            return Err(serde::de::Error::invalid_length(len, &""));
        }
        let mut heights = [[0.0; RESOLUTION]; RESOLUTION];
        for row in &mut heights {
            for height in row {
                *height = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &""))?;
            }
        }
        Ok(heights)
    }
}
