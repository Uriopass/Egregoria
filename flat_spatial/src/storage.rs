use crate::Vec2;

pub type CellIdx = (i32, i32);

pub(crate) fn cell_range(
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

/// `SparseStorage` stores cells in a `FastMap` to be used in a Grid.
/// It is Sparse because cells are eagerly allocated, and cleaned when they are empty.
/// It implements the Storage trait.
#[derive(Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparseStorage<T: Default> {
    cell_size: i32,
    cells: fnv::FnvHashMap<CellIdx, T>,
}

impl<T: Default> SparseStorage<T> {
    pub(crate) fn new(cell_size: i32) -> Self {
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

    pub(crate) fn cell_size(&self) -> i32 {
        self.cell_size
    }

    pub(crate) fn modify(&mut self, mut f: impl FnMut(&mut T) -> bool) {
        self.cells.retain(move |_, cell| !f(cell));
    }

    pub(crate) fn cell_mut<V2: Vec2>(&mut self, pos: V2) -> (CellIdx, &mut T) {
        let id = self.cell_id(pos);
        (id, self.cells.entry(id).or_default())
    }

    pub(crate) fn cell_mut_unchecked(&mut self, id: CellIdx) -> &mut T {
        self.cells.entry(id).or_default()
    }

    pub(crate) fn cell(&self, id: CellIdx) -> Option<&T> {
        self.cells.get(&id)
    }

    pub(crate) fn cell_id<V2: Vec2>(&self, pos: V2) -> CellIdx {
        (
            pos.x() as i32 / self.cell_size - if pos.x() < 0.0 { 1 } else { 0 },
            pos.y() as i32 / self.cell_size - if pos.y() < 0.0 { 1 } else { 0 },
        )
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
