use crate::{vec2, vec3, Ray3, Vec2, Vec3, AABB, AABB3};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};

pub type HeightmapChunkID = (u16, u16);

const MIN_HEIGHT: f32 = -40.0;
const MAX_HEIGHT: f32 = 2008.0;

#[derive(Clone)]
pub struct HeightmapChunk<const RESOLUTION: usize, const SIZE: u32> {
    heights: [[f32; RESOLUTION]; RESOLUTION], // TODO: change to RESOLUTION * RESOLUTION when generic_const_exprs is stabilized
    max_height: f32,
}

impl<const RESOLUTION: usize, const SIZE: u32> Default for HeightmapChunk<RESOLUTION, SIZE> {
    fn default() -> Self {
        Self {
            heights: [[0.0; RESOLUTION]; RESOLUTION],
            max_height: 0.0,
        }
    }
}

impl<const RESOLUTION: usize, const SIZE: u32> HeightmapChunk<RESOLUTION, SIZE> {
    pub fn new(heights: [[f32; RESOLUTION]; RESOLUTION]) -> Self {
        let mut max_height = heights[0][0];
        for row in &heights {
            for height in row {
                max_height = max_height.max(*height);
            }
        }

        Self {
            heights,
            max_height,
        }
    }

    #[inline]
    pub fn rect(id: HeightmapChunkID) -> AABB {
        let ll = vec2(id.0 as f32 * SIZE as f32, id.1 as f32 * SIZE as f32);
        let ur = ll + vec2(SIZE as f32, SIZE as f32);
        AABB::new(ll, ur)
    }

    #[inline]
    pub fn id(v: Vec2) -> HeightmapChunkID {
        let x = v.x / SIZE as f32;
        let x = x.clamp(0.0, u16::MAX as f32) as u16;
        let y = v.y / SIZE as f32;
        let y = y.clamp(0.0, u16::MAX as f32) as u16;
        (x, y)
    }

    #[inline]
    pub fn bbox(&self, origin: Vec2) -> AABB3 {
        AABB3::new(
            vec3(origin.x, origin.y, MIN_HEIGHT),
            vec3(
                origin.x + SIZE as f32,
                origin.y + SIZE as f32,
                self.max_height,
            ),
        )
    }

    /// assume p is in chunk-space and in-bounds
    #[inline]
    pub fn height_unchecked(&self, p: Vec2) -> f32 {
        let v = p / SIZE as f32;
        let v = v * RESOLUTION as f32;
        self.heights[v.y as usize][v.x as usize]
    }

    #[inline]
    pub fn height(&self, p: Vec2) -> Option<f32> {
        let v = p / SIZE as f32;
        let v = v * RESOLUTION as f32;
        self.heights.get(v.y as usize)?.get(v.x as usize).copied()
    }

    #[inline]
    pub fn heights(&self) -> &[[f32; RESOLUTION]; RESOLUTION] {
        &self.heights
    }

    #[inline]
    pub fn max_height(&self) -> f32 {
        self.max_height
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

    #[inline]
    pub fn bounds(&self) -> AABB {
        AABB::new(
            vec2(0.0, 0.0),
            vec2(self.w as f32 * SIZE as f32, self.h as f32 * SIZE as f32),
        )
    }

    #[inline]
    fn check_valid(&self, id: HeightmapChunkID) -> bool {
        id.0 < self.w && id.1 < self.h
    }

    #[inline]
    pub fn set_chunk(&mut self, id: HeightmapChunkID, chunk: HeightmapChunk<RESOLUTION, SIZE>) {
        if !self.check_valid(id) {
            return;
        }
        self.chunks[(id.0 + id.1 * self.w) as usize] = chunk;
    }

    #[inline]
    pub fn get_chunk(&self, id: HeightmapChunkID) -> Option<&HeightmapChunk<RESOLUTION, SIZE>> {
        if !self.check_valid(id) {
            return None;
        }
        unsafe { Some(self.chunks.get_unchecked((id.0 + id.1 * self.w) as usize)) }
    }

    fn get_chunk_mut(
        &mut self,
        id: HeightmapChunkID,
    ) -> Option<&mut HeightmapChunk<RESOLUTION, SIZE>> {
        if !self.check_valid(id) {
            return None;
        }
        unsafe {
            Some(
                self.chunks
                    .get_unchecked_mut((id.0 + id.1 * self.w) as usize),
            )
        }
    }

    /// Applies a function to every point in the heightmap in the given bounds
    pub fn apply(&mut self, bounds: AABB, mut f: impl FnMut(Vec3) -> f32) -> Vec<HeightmapChunkID> {
        let ll = bounds.ll / SIZE as f32;
        let ur = bounds.ur / SIZE as f32;
        let ll = vec2(ll.x.floor(), ll.y.floor());
        let ur = vec2(ur.x.ceil(), ur.y.ceil());

        let mut modified = Vec::with_capacity(((ur.x - ll.x) * (ur.y - ll.y)) as usize);

        for x in ll.x as u16..ur.x as u16 {
            for y in ll.y as u16..ur.y as u16 {
                let id = (x, y);
                let Some(chunk) = self.get_chunk_mut(id) else {
                    continue;
                };
                modified.push(id);
                let corner = vec2(x as f32, y as f32) * SIZE as f32;
                let mut max_height: f32 = 0.0;
                for i in 0..RESOLUTION {
                    for j in 0..RESOLUTION {
                        let p = corner + vec2(j as f32, i as f32) * Self::CELL_SIZE;
                        let h = chunk.heights[i][j];
                        max_height = max_height.max(h);
                        if !bounds.contains(p) {
                            continue;
                        }
                        let new_h = f(p.z(h)).clamp(MIN_HEIGHT, MAX_HEIGHT);
                        chunk.heights[i][j] = new_h;
                        max_height = max_height.max(new_h);
                    }
                }
                chunk.max_height = max_height;
            }
        }

        modified
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
            let x = (p.x / Self::CELL_SIZE).fract();
            let y = (p.y / Self::CELL_SIZE).fract();

            let h01 = ll + x * (lr - ll);
            let h23 = ul + x * (ur - ul);

            return Some(h01 + y * (h23 - h01));
        }
        exact
    }

    /// Casts a ray on the heightmap, returning the point of intersection and the normal at that point
    /// We assume height is between [-40.0; 2008]
    pub fn raycast(&self, ray: Ray3) -> Option<(Vec3, Vec3)> {
        // Let's build an iterator over the chunks that intersect the ray (from nearest to furthest)
        let start = ray.from.xy() / SIZE as f32;
        let end = start + ray.dir.xy().normalize() * self.w.max(self.h) as f32 * 2.0;

        let diff = end - start;
        let l = diff.mag();
        let speed = diff / l;

        let mut t = 0.0;

        let mut cur = start;

        let intersecting_chunks = std::iter::once((start.x as isize, start.y as isize))
            .chain(std::iter::from_fn(|| {
                let x = cur.x - cur.x.floor();
                let y = cur.y - cur.y.floor();

                let t_x;
                let t_y;

                if speed.x >= 0.0 {
                    t_x = (1.0 - x) / speed.x;
                } else {
                    t_x = -x / speed.x;
                }
                if speed.y >= 0.0 {
                    t_y = (1.0 - y) / speed.y;
                } else {
                    t_y = -y / speed.y;
                }

                let min_t = t_x.min(t_y) + 0.0001;
                t += min_t;
                if !(t < l) {
                    // reverse the condition to avoid infinite loop in case of NaN
                    return None;
                }
                cur += min_t * speed;
                Some((cur.x as isize, cur.y as isize))
            }))
            .filter(|&(x, y)| x < self.w as isize && y < self.h as isize && x >= 0 && y >= 0)
            .filter_map(|(x, y)| {
                let chunk_id = (x as u16, y as u16);
                let corner = vec2(x as f32, y as f32) * SIZE as f32;
                let (t_min, t_max) = self.get_chunk(chunk_id)?.bbox(corner).raycast(ray)?;
                Some((t_min, t_max))
            });

        // Now within those chunks, let's try to find the intersection point
        // h < t * ray.dir.z + ray.from.z
        for (t_min, t_max) in intersecting_chunks {
            let mut t = t_min;
            let t_step = Self::CELL_SIZE;

            loop {
                let p = ray.from + ray.dir * t;
                let Some(h) = self.height(p.xy()) else {
                    if t >= t_max {
                        break;
                    }
                    t += t_step;
                    continue;
                };
                if p.z < h {
                    // we found a good candidate but we're not there yet
                    // we still need to do one last binary search
                    // to find the bilinear-filtered-corrected location

                    let t = binary_search(t - t_step * 2.0, t, |t| {
                        let p = ray.from + ray.dir * t;
                        let Some(h) = self.height(p.xy()) else {
                            return false;
                        };
                        p.z < h
                    });

                    return Some((ray.from + ray.dir * t, vec3(0.0, 0.0, 1.0)));
                }
                if t >= t_max {
                    break;
                }
                t += t_step;
            }
        }

        None
    }
}

/// Does a binary search on the interval [min; max] to find the first value for which f returns true
fn binary_search(min: f32, max: f32, mut f: impl FnMut(f32) -> bool) -> f32 {
    let mut min = min;
    let mut max = max;
    let mut mid = min + (max - min) * 0.5;
    loop {
        if f(mid) {
            max = mid;
        } else {
            min = mid;
        }
        let new_mid = min + (max - min) * 0.5;
        if (new_mid - mid).abs() < 0.0001 {
            break;
        }
        mid = new_mid;
    }
    mid
}

impl<const RESOLUTION: usize, const SIZE: u32> Serialize for HeightmapChunk<RESOLUTION, SIZE> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(1 + RESOLUTION * RESOLUTION))?;
        seq.serialize_element(&self.max_height)?;
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
        let (heights, max_height) =
            deserializer.deserialize_seq(HeightmapChunkVisitor::<RESOLUTION>)?;
        Ok(Self {
            heights,
            max_height,
        })
    }
}

struct HeightmapChunkVisitor<const RESOLUTION: usize>;

impl<'de, const RESOLUTION: usize> serde::de::Visitor<'de> for HeightmapChunkVisitor<RESOLUTION> {
    type Value = ([[f32; RESOLUTION]; RESOLUTION], f32);

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of floats")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let len = seq.size_hint().unwrap_or(1 + RESOLUTION * RESOLUTION);
        if len != 1 + RESOLUTION * RESOLUTION {
            return Err(serde::de::Error::invalid_length(len, &""));
        }
        let max_height = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &""))?;
        let mut heights = [[0.0; RESOLUTION]; RESOLUTION];
        for row in &mut heights {
            for height in row {
                *height = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &""))?;
            }
        }
        Ok((heights, max_height))
    }
}
