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
        let ll = vec2(ll.x.floor(), ll.y.floor()).max(Vec2::ZERO);
        let ur = vec2(ur.x.ceil(), ur.y.ceil()).min(vec2(self.w as f32, self.h as f32));

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

    /// Applies a convolution to every point in the heightmap in the given bounds
    /// The function f is called with the point location and the 3x3 grid of heights around it, indexed with (x + y * 3)
    /// This operation is much slower than apply because a copy of the chunk must be done
    pub fn apply_convolution(
        &mut self,
        bounds: AABB,
        mut f: impl FnMut(Vec2, [f32; 9]) -> f32,
    ) -> Vec<HeightmapChunkID> {
        let ll = bounds.ll / SIZE as f32;
        let ur = bounds.ur / SIZE as f32;
        let ll = vec2(ll.x.floor(), ll.y.floor()).max(Vec2::ZERO);
        let ur = vec2(ur.x.ceil(), ur.y.ceil()).min(vec2(self.w as f32, self.h as f32));

        let mut modified = Vec::with_capacity(((ur.x - ll.x) * (ur.y - ll.y)) as usize);

        let mut new_chunks = Vec::with_capacity(((ur.x - ll.x) * (ur.y - ll.y)) as usize);
        for y in ll.y as u16..ur.y as u16 {
            for x in ll.x as u16..ur.x as u16 {
                let id = (x, y);
                modified.push(id);
                let corner = vec2(x as f32, y as f32) * SIZE as f32;
                let mut max_height: f32 = 0.0;
                let mut new_heights = [[0.0; RESOLUTION]; RESOLUTION];
                for i in 0..RESOLUTION {
                    for j in 0..RESOLUTION {
                        let p = corner + vec2(j as f32, i as f32) * Self::CELL_SIZE;
                        let mut conv_heights = [0.0; 9];
                        for conv_y in 0..3 {
                            for conv_x in 0..3 {
                                conv_heights[conv_x + conv_y * 3] = self
                                    .height_idx(
                                        (x as usize * RESOLUTION + j + conv_x).saturating_sub(1),
                                        (y as usize * RESOLUTION + i + conv_y).saturating_sub(1),
                                    )
                                    .unwrap_or(0.0);
                            }
                        }
                        let mut new_h = conv_heights[4];
                        if bounds.contains(p) {
                            new_h = f(p, conv_heights).clamp(MIN_HEIGHT, MAX_HEIGHT);
                        }
                        new_heights[i][j] = new_h;
                        max_height = max_height.max(new_h);
                    }
                }
                new_chunks.push((id, new_heights, max_height));
            }
        }

        for (id, new_heights, max_height) in new_chunks {
            let chunk = self.get_chunk_mut(id).unwrap();
            chunk.heights = new_heights;
            chunk.max_height = max_height;
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

    /// get height by actual cell position
    #[inline]
    pub fn height_idx(&self, x: usize, y: usize) -> Option<f32> {
        let chunkx = x / RESOLUTION;
        let chunky = y / RESOLUTION;

        let cellx = x % RESOLUTION;
        let celly = y % RESOLUTION;

        let chunk = self.get_chunk((chunkx as u16, chunky as u16))?;
        Some(chunk.heights[celly][cellx])
    }

    pub fn height_idx_mut(&mut self, x: usize, y: usize) -> Option<&mut f32> {
        let chunkx = x / RESOLUTION;
        let chunky = y / RESOLUTION;

        let cellx = x % RESOLUTION;
        let celly = y % RESOLUTION;

        let chunk = self.get_chunk_mut((chunkx as u16, chunky as u16))?;
        Some(&mut chunk.heights[celly][cellx])
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

    /// Returns height and gradient at any point using bilinear interpolation
    /// The gradient is the vector pointing in the direction of the steepest slope (downwards)
    pub fn height_gradient(&self, p: Vec2) -> Option<(f32, Vec2)> {
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

            let height = h01 + y * (h23 - h01);
            let gradient = -vec2(
                (lr - ll) + y * ((ur - ul) - (lr - ll)),
                (ul - ll) + x * ((ur - lr) - (ul - ll)),
            );

            return Some((height, gradient));
        }
        exact.zip(Some(Vec2::ZERO))
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

fn pack_height(height: f32) -> u16 {
    ((height - MIN_HEIGHT) / (MAX_HEIGHT - MIN_HEIGHT) * u16::MAX as f32) as u16
}

fn unpack_height(height: u16) -> f32 {
    height as f32 / u16::MAX as f32 * (MAX_HEIGHT - MIN_HEIGHT) + MIN_HEIGHT
}

impl<const RESOLUTION: usize, const SIZE: u32> Serialize for HeightmapChunk<RESOLUTION, SIZE> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(1 + RESOLUTION * RESOLUTION))?;
        seq.serialize_element(&self.max_height)?;
        let mut last = 0;
        for row in &self.heights {
            for &height in row {
                let packed = pack_height(height);
                let delta = packed.wrapping_sub(last);
                seq.serialize_element(&delta)?;
                last = packed;
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
        let mut last = 0;
        for row in &mut heights {
            for height in row {
                let delta: u16 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &""))?;
                let packed = delta.wrapping_add(last);
                *height = unpack_height(packed);
                last = packed;
            }
        }
        Ok((heights, max_height))
    }
}

mod erosion {
    use crate::heightmap::MIN_HEIGHT;
    use crate::{vec2, Heightmap, HeightmapChunk, HeightmapChunkID, Radians, Vec2, AABB};
    use std::collections::BTreeSet;
    use std::ops::Div;

    // taken from https://github.com/SebLague/Hydraulic-Erosion/blob/master/Assets/Scripts/Erosion.cs
    // Copyright (c) 2019 Sebastian Lague
    // 2..8
    const EROSION_RADIUS: isize = 3;
    // 0..1
    const INERTIA: f32 = 0.1; // At zero, water will instantly change direction to flow downhill. At 1, water will never change direction.
    const SEDIMENT_CAPACITY_FACTOR: f32 = 1.0; // Multiplier for how much sediment a droplet can carry
    const MIN_SEDIMENT_CAPACITY: f32 = 0.003; // Used to prevent carry capacity getting too close to zero on flatter terrain

    // 0..1
    const ERODE_SPEED: f32 = 0.3;

    // 0..1
    const DEPOSIT_SPEED: f32 = 0.3;
    // 0..1
    const EVAPORATE_SPEED: f32 = 0.01;
    const GRAVITY: f32 = 1.0;
    const MAX_DROPLET_LIFETIME: usize = 50;

    const INITIAL_WATER_VOLUME: f32 = 1.0;
    const INITIAL_SPEED: f32 = 1.0;

    impl<const RESOLUTION: usize, const SIZE: u32> Heightmap<RESOLUTION, SIZE> {
        #[rustfmt::skip]
        pub fn erode(
            &mut self,
            bounds: AABB,
            n_particles: usize,
            mut randgen: impl FnMut() -> f32,
        ) -> Vec<HeightmapChunkID> {
            let mut changed = BTreeSet::new();

            let mut erosion_brush_total = 0.0;
            for y in -EROSION_RADIUS..=EROSION_RADIUS {
                for x in -EROSION_RADIUS..=EROSION_RADIUS {
                    let dist2 = x * x + y * y;
                    if dist2 >= EROSION_RADIUS * EROSION_RADIUS {
                        continue;
                    }
                    erosion_brush_total += 1.0 - f32::sqrt(dist2 as f32) / EROSION_RADIUS as f32;
                }
            }

            for _ in 0..n_particles {
                // Create water droplet at random point in bounds, in a circle

                let d = randgen().powf(0.8) * (bounds.size() / 2.0).mag();
                let angle = Radians(randgen() * std::f32::consts::TAU);
                let pos = bounds.center() + angle.vec2() * d;


                let mut pos = pos / Self::CELL_SIZE;

                let mut dir = Vec2::ZERO;

                let mut speed = INITIAL_SPEED;
                let mut water = INITIAL_WATER_VOLUME;
                let mut sediment = 0.0;

                for _ in 0..MAX_DROPLET_LIFETIME {
                    //dbg!((pos, dir, speed, water, sediment,));
                    //unsafe {
                    //    let size = 10.0 * (MAX_DROPLET_LIFETIME as f32 - i as f32)
                    //        / MAX_DROPLET_LIFETIME as f32;
                    //    DEBUG_OBBS.push(OBB::new(pos * Self::CELL_SIZE, Vec2::X, size, size));
                    //}
                    let cell_offset = pos - pos.floor();

                    // Calculate droplet's height and direction of flow with bilinear interpolation of surrounding heights
                    let Some((height, gradient)) = self.height_gradient(pos * Self::CELL_SIZE)
                    else {
                        break;
                    };

                    // Update the droplet's direction and position (move position 1 unit regardless of speed)
                    dir = Vec2::lerp(gradient, dir, INERTIA);

                    // Normalize direction
                    let dl = dir.mag();
                    if dl < f32::EPSILON {
                        dir = Vec2::from_angle(Radians(randgen() * std::f32::consts::TAU));
                    } else {
                        dir /= dl;
                    }

                    // Stop simulating droplet if it's not moving or has flowed over edge of map
                    if (dir.x == 0.0 && dir.y == 0.0)
                        || pos.x < 0.0
                        || pos.x >= self.w as f32 * RESOLUTION as f32
                        || pos.y < 0.0
                        || pos.y >= self.h as f32 * RESOLUTION as f32
                    {
                        break;
                    }

                    // Find the droplet's new height and calculate the deltaHeight
                    let new_height = self.height((pos + dir) * Self::CELL_SIZE).unwrap_or(0.0);
                    let delta_height = (new_height - height) / Self::CELL_SIZE;

                    // Calculate the droplet's sediment capacity (higher when moving fast down a slope and contains lots of water)
                    let sediment_capacity =
                        (-delta_height * speed * water * SEDIMENT_CAPACITY_FACTOR)
                            .max(MIN_SEDIMENT_CAPACITY);

                    // If carrying more sediment than capacity, or if flowing uphill:
                    if sediment > sediment_capacity || delta_height > 0.0 {
                        // If moving uphill (delta_height > 0) try fill up to the current height, otherwise deposit a fraction of the excess sediment
                        let amount_to_deposit = if delta_height > 0.0 {
                            f32::min(delta_height, sediment)
                        } else {
                            (sediment - sediment_capacity) * DEPOSIT_SPEED
                        };
                        sediment -= amount_to_deposit;

                        // Add the sediment to the four nodes of the current cell using bilinear interpolation
                        // Deposition is not distributed over a radius (like erosion) so that it can fill small pits
                        changed.insert(HeightmapChunk::<RESOLUTION, SIZE>::id(
                            pos * Self::CELL_SIZE,
                        ));

                        let amount_to_deposit = amount_to_deposit * Self::CELL_SIZE;
                        {
                            self.height_idx_mut(pos.x as usize    , pos.y as usize)    .map(|v| { *v += amount_to_deposit * (1.0 - cell_offset.x) * (1.0 - cell_offset.y) });
                            self.height_idx_mut(pos.x as usize + 1, pos.y as usize)    .map(|v| { *v += amount_to_deposit * cell_offset.x         * (1.0 - cell_offset.y) });
                            self.height_idx_mut(pos.x as usize    , pos.y as usize + 1).map(|v| { *v += amount_to_deposit * (1.0 - cell_offset.x) * cell_offset.y });
                            self.height_idx_mut(pos.x as usize + 1, pos.y as usize + 1).map(|v|   *v += amount_to_deposit * cell_offset.x         * cell_offset.y);
                        }
                    } else {
                        // Erode a fraction of the droplet's current carry capacity.
                        // Clamp the erosion to the change in height so that it doesn't dig a hole in the terrain behind the droplet
                        let amount_to_erode =
                            f32::min((sediment_capacity - sediment) * ERODE_SPEED, -delta_height);

                        // Use erosion brush to erode from all nodes inside the droplet's erosion radius
                        changed.insert(HeightmapChunk::<RESOLUTION, SIZE>::id(pos * Self::CELL_SIZE));

                        for erode_y in -EROSION_RADIUS..=EROSION_RADIUS {
                            for erode_x in -EROSION_RADIUS..=EROSION_RADIUS {
                                let dist2 = erode_x * erode_x + erode_y * erode_y;
                                if dist2 >= EROSION_RADIUS * EROSION_RADIUS {
                                    continue;
                                }
                                let w = (1.0 - f32::sqrt(dist2 as f32) / EROSION_RADIUS as f32) / erosion_brush_total;

                                let pos_radius = pos + vec2(erode_x as f32, erode_y as f32);

                                let weighed_erode_amount = amount_to_erode * w;

                                let delta_sediment = (self
                                    .height_idx(pos_radius.x as usize, pos_radius.y as usize)
                                    .unwrap_or(0.0) - MIN_HEIGHT)
                                    .div(Self::CELL_SIZE)
                                    .min(weighed_erode_amount);

                                self.height_idx_mut(pos_radius.x as usize, pos_radius.y as usize)
                                    .map(|v| *v -= delta_sediment * Self::CELL_SIZE);

                                //dbg!(delta_sediment, weighed_erode_amount, amount_to_erode);
                                sediment += delta_sediment;
                            }
                        }
                    }

                    // Update droplet's speed and water content
                    speed = f32::sqrt(speed * speed - delta_height * GRAVITY) * 0.98;
                    water *= 1.0 - EVAPORATE_SPEED;
                    pos += dir;
                }
            }

            changed.into_iter().collect()
        }
    }
}
