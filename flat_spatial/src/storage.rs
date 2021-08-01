use common::FastMap;
use geom::{Vec2, AABB};
use serde::{Deserialize, Serialize};

pub type CellIdx = (i32, i32);

pub fn cell_range(
    (min_x, min_y): CellIdx,
    (max_x, max_y): CellIdx,
) -> impl Iterator<Item = CellIdx> {
    if min_x > max_x || min_y > max_y {
        return XYRange {
            min_x: 0,
            max_x: 0,
            max_y: 0,
            x: 1,
            y: 1,
        };
    }
    XYRange {
        min_x,
        max_x,
        max_y,
        x: min_x,
        y: min_y,
    }
}

/// The storage trait, implement this if you want to use a custom point storage for the Grid.
pub trait Storage<T> {
    fn new(cell_size: i32) -> Self;
    fn cell_size(&self) -> i32;

    fn modify(&mut self, f: impl FnMut(&mut T) -> bool);

    fn cell_mut(&mut self, pos: Vec2) -> (CellIdx, &mut T);

    fn cell_mut_unchecked(&mut self, id: CellIdx) -> &mut T;
    fn cell(&self, id: CellIdx) -> Option<&T>;

    fn cell_id(&self, p: Vec2) -> CellIdx;

    fn cell_aabb(&self, id: CellIdx) -> AABB;
}

/// `DenseStorage` stores cells in a Vec to be used for a Grid.
/// It implements the Storage trait.
#[derive(Clone, Deserialize, Serialize)]
pub struct DenseStorage<T: Default> {
    cell_size: i32,
    start_x: i32,
    start_y: i32,
    width: i32,
    height: i32,
    cells: Vec<T>,
}

impl<T: Default> DenseStorage<T> {
    /// Creates a new cell grid centered on zero with width and height defined by size.  
    ///
    /// Note that the size is counted in cells and not in absolute units (!)
    pub fn new_centered(cell_size: i32, size: i32) -> Self {
        Self::new_rect(cell_size, -size, -size, 2 * size, 2 * size)
    }

    /// Creates a new grid with a custom rect defining its boundaries.  
    ///
    /// Note that the coordinates are counted in cells and not in absolute units (!)
    pub fn new_rect(cell_size: i32, x: i32, y: i32, w: i32, h: i32) -> Self {
        assert!(
            cell_size > 0,
            "Cell size ({}) cannot be less than or equal to zero",
            cell_size
        );
        Self {
            start_x: x,
            start_y: y,
            cell_size,
            width: w,
            height: h,
            cells: (0..w * h).map(|_| Default::default()).collect(),
        }
    }

    pub fn cells(&self) -> &Vec<T> {
        &self.cells
    }

    fn pos(&self, (x, y): CellIdx) -> usize {
        ((y - self.start_y) * self.width + (x - self.start_x)) as usize
    }
}

impl<T: Default> Storage<T> for DenseStorage<T> {
    fn new(cell_size: i32) -> Self {
        Self {
            cell_size,
            start_x: 0,
            start_y: 0,
            width: 0,
            height: 0,
            cells: vec![],
        }
    }

    fn cell_size(&self) -> i32 {
        self.cell_size
    }

    fn modify(&mut self, mut f: impl FnMut(&mut T) -> bool) {
        self.cells.iter_mut().for_each(|x| {
            f(x);
        })
    }

    fn cell_mut(&mut self, pos: Vec2) -> (CellIdx, &mut T) {
        debug_assert!(pos.x.is_finite());
        debug_assert!(pos.y.is_finite());

        if self.width == 0 && self.height == 0 {
            // First allocation, change start_x and start_y to match pos
            self.start_x = pos.x as i32 / self.cell_size;
            self.start_y = pos.y as i32 / self.cell_size;
            self.width = 1;
            self.height = 1;
            self.cells = vec![T::default()];
        }
        let mut reallocate = false;

        let mut padleft = 0;
        let mut padright = 0;
        let mut paddown = 0;
        let mut padup = 0;

        let x = pos.x as i32;
        let y = pos.y as i32;

        let left = self.start_x * self.cell_size;
        let down = self.start_y * self.cell_size;
        let right = left + self.width * self.cell_size;
        let up = down + self.height * self.cell_size;

        if x <= left {
            padleft = 1 + (left - x) / self.cell_size;
            self.start_x -= padleft;
            self.width += padleft;
            reallocate = true;
        } else if x >= right {
            padright = 1 + (x - right) / self.cell_size;
            self.width += padright;
            reallocate = true;
        }

        if y <= down {
            paddown = 1 + (down - y) / self.cell_size;
            self.start_y -= paddown;
            self.height += paddown;
            reallocate = true;
        } else if y >= up {
            padup = 1 + (y - up) / self.cell_size;
            self.height += padup;
            if !reallocate {
                self.cells
                    .resize_with((self.width * self.height) as usize, T::default);
            }
        }

        if reallocate {
            let mut newvec = Vec::with_capacity((self.width * self.height) as usize);

            let oldh = self.height - paddown - padup;
            let oldw = self.width - padleft - padright;

            // use T::default to pad with new cells
            for _ in 0..paddown {
                newvec.extend((0..self.width).map(|_| T::default()))
            }
            for y in 0..oldh {
                newvec.extend((0..padleft).map(|_| T::default()));
                newvec.extend(
                    (0..oldw).map(|x| {
                        std::mem::take(self.cells.get_mut((y * oldw + x) as usize).unwrap())
                    }),
                );
                newvec.extend((0..padright).map(|_| T::default()))
            }
            for _ in 0..padup {
                newvec.extend((0..self.width).map(|_| T::default()))
            }

            self.cells = newvec;
        }

        let id = self.cell_id(pos);
        (id, self.cell_mut_unchecked(id))
    }

    fn cell_mut_unchecked(&mut self, id: CellIdx) -> &mut T {
        let p = self.pos(id);
        &mut self.cells[p]
    }

    fn cell(&self, id: CellIdx) -> Option<&T> {
        self.cells.get(self.pos(id))
    }

    fn cell_id(&self, pos: Vec2) -> CellIdx {
        (
            pos.x as i32 / self.cell_size - if pos.x < 0.0 { 1 } else { 0 },
            pos.y as i32 / self.cell_size - if pos.y < 0.0 { 1 } else { 0 },
        )
    }

    fn cell_aabb(&self, id: CellIdx) -> AABB {
        let (x, y) = id;

        let ll = Vec2 {
            x: (x * self.cell_size) as f32,
            y: (y * self.cell_size) as f32,
        };

        let ur = Vec2 {
            x: ll.x + self.cell_size as f32,
            y: ll.y + self.cell_size as f32,
        };

        AABB::new(ll, ur)
    }
}

/// `SparseStorage` stores cells in a `FastMap` to be used in a Grid.
/// It is Sparse because cells are eagerly allocated, and cleaned when they are empty.
/// It implements the Storage trait.
#[derive(Clone, Deserialize, Serialize)]
pub struct SparseStorage<T: Default> {
    cell_size: i32,
    cells: FastMap<CellIdx, T>,
}

impl<T: Default> SparseStorage<T> {
    pub fn cells(&self) -> &FastMap<CellIdx, T> {
        &self.cells
    }
}

impl<T: Default> Storage<T> for SparseStorage<T> {
    fn new(cell_size: i32) -> Self {
        assert!(
            cell_size > 0,
            "Cell size ({}) cannot be less than or equal to zero",
            cell_size
        );
        Self {
            cell_size,
            cells: Default::default(),
        }
    }

    fn cell_size(&self) -> i32 {
        self.cell_size
    }

    fn modify(&mut self, mut f: impl FnMut(&mut T) -> bool) {
        self.cells.retain(move |_, cell| !f(cell));
    }

    fn cell_mut(&mut self, pos: Vec2) -> (CellIdx, &mut T) {
        let id = self.cell_id(pos);
        (id, self.cells.entry(id).or_default())
    }

    fn cell_mut_unchecked(&mut self, id: CellIdx) -> &mut T {
        self.cells.entry(id).or_default()
    }

    fn cell(&self, id: CellIdx) -> Option<&T> {
        self.cells.get(&id)
    }

    fn cell_id(&self, pos: Vec2) -> CellIdx {
        (
            pos.x as i32 / self.cell_size - if pos.x < 0.0 { 1 } else { 0 },
            pos.y as i32 / self.cell_size - if pos.y < 0.0 { 1 } else { 0 },
        )
    }

    fn cell_aabb(&self, (x, y): CellIdx) -> AABB {
        let ll = Vec2 {
            x: (x * self.cell_size) as f32,
            y: (y * self.cell_size) as f32,
        };

        let ur = Vec2 {
            x: ll.x + self.cell_size as f32,
            y: ll.y + self.cell_size as f32,
        };

        AABB::new(ll, ur)
    }
}

pub struct XYRange {
    min_x: i32,
    max_x: i32,
    max_y: i32,
    x: i32,
    y: i32,
}

impl Iterator for XYRange {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y > self.max_y {
            return None;
        }

        let v = (self.x, self.y);
        self.x += 1;
        if self.x > self.max_x {
            self.x = self.min_x;
            self.y += 1;
        }

        Some(v)
    }
}
