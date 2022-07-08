use crate::cell::ShapeGridCell;
use crate::storage::{cell_range, SparseStorage};
use crate::AABB;
use slotmap::{new_key_type, SlotMap};

pub type ShapeGridObjects<O, AB> = SlotMap<ShapeGridHandle, StoreObject<O, AB>>;

new_key_type! {
    /// This handle is used to modify the associated object or to update its position.
    /// It is returned by the _insert_ method of a ShapeGrid.
    pub struct ShapeGridHandle;
}

/// The actual object stored in the store
#[derive(Clone, Copy)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StoreObject<O: Copy, AB: AABB> {
    /// User-defined object to be associated with a value
    pub obj: O,
    pub aabb: AB,
}

/// `ShapeGrid` is a generic aabb-based spatial partitioning structure that uses a generic storage of cells which acts as a
/// grid instead of a tree.
///
/// ## Fast queries
/// In theory, `ShapeGrid` should be faster than a quadtree/r-tree because it has no log costs
/// (calculating the cells around a point is trivial).  
/// However, it only works if the cell size is adapted to the problem, much like how a tree has to
/// be balanced to be efficient.  
///
/// ## Dynamicity
/// `ShapeGrid's` allows eager removals and position updates, however for big aabbs (spanning many cells)
/// this can be expensive, so beware.
///
/// Use this grid for mostly static objects with the occasional removal/position update if needed.
///
/// A `SlotMap` is used for objects managing, adding a level of indirection between aabbs and objects.
/// `SlotMap` is used because removal doesn't alter handles given to the user, while still having constant time access.
/// However it requires O to be copy, but `SlotMap's` author stated that they were working on a similar
/// map where Copy isn't required.
///
/// ## About object management
///
/// In theory, you don't have to use the object management directly, you can make your custom
/// Handle -> Object map by specifying "`()`" to be the object type.
/// _(This can be useful if your object is not Copy)_
/// Since `()` is zero sized, it should probably optimize away a lot of the object management code.
///
/// ```rust
/// use flat_spatial::AABBGrid;
/// use euclid::default::Rect;
///
/// let mut g: AABBGrid<(), Rect<f32>> = AABBGrid::new(10);
/// let handle = g.insert(Rect::new([0.0, 0.0].into(), [10.0, 10.0].into()), ());
/// // Use handle however you want
/// ```
#[derive(Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AABBGrid<O: Copy, AB: AABB> {
    storage: SparseStorage<ShapeGridCell>,
    objects: ShapeGridObjects<O, AB>,
}

impl<O: Copy, AB: AABB> AABBGrid<O, AB> {
    /// Creates an empty grid.
    /// The cell size should be about the same magnitude as your queries size.
    pub fn new(cell_size: i32) -> Self {
        Self {
            storage: SparseStorage::new(cell_size),
            objects: ShapeGridObjects::default(),
        }
    }

    /// Clears the grid.
    pub fn clear(&mut self) -> impl Iterator<Item = (AB, O)> {
        self.storage = SparseStorage::new(self.storage.cell_size());
        let objs = std::mem::take(&mut self.objects);
        objs.into_iter().map(|(_, o)| (o.aabb, o.obj))
    }

    /// Inserts a new object with a position and an associated object
    /// Returns the unique and stable handle to be used with `get_obj`
    pub fn insert(&mut self, aabb: AB, obj: O) -> ShapeGridHandle {
        let Self {
            storage, objects, ..
        } = self;

        let h = objects.insert(StoreObject { obj, aabb });
        cells_apply(storage, &aabb, |cell, sing_cell| {
            cell.objs.push((h, sing_cell));
        });
        h
    }

    /// Updates the aabb of an object.
    pub fn set_aabb(&mut self, handle: ShapeGridHandle, aabb: AB) {
        let obj = self
            .objects
            .get_mut(handle)
            .expect("Object not in grid anymore");

        let storage = &mut self.storage;

        cells_apply(storage, &obj.aabb, |cell, _| {
            let p = match cell.objs.iter().position(|(x, _)| *x == handle) {
                Some(x) => x,
                None => return,
            };
            cell.objs.swap_remove(p);
        });

        cells_apply(storage, &aabb, |cell, sing_cell| {
            cell.objs.push((handle, sing_cell))
        });

        obj.aabb = aabb;
    }

    /// Removes an object from the grid.
    pub fn remove(&mut self, handle: ShapeGridHandle) -> Option<O> {
        let st = self.objects.remove(handle)?;

        let storage = &mut self.storage;
        cells_apply(storage, &st.aabb, |cell, _| {
            let p = match cell.objs.iter().position(|(x, _)| *x == handle) {
                Some(x) => x,
                None => return,
            };
            cell.objs.swap_remove(p);
        });

        Some(st.obj)
    }

    /// Iterate over all handles
    pub fn handles(&self) -> impl Iterator<Item = ShapeGridHandle> + '_ {
        self.objects.keys()
    }

    /// Iterate over all objects
    pub fn objects(&self) -> impl Iterator<Item = &O> + '_ {
        self.objects.values().map(|x| &x.obj)
    }

    /// Returns a reference to the associated object and its position, using the handle.
    pub fn get(&self, id: ShapeGridHandle) -> Option<&StoreObject<O, AB>> {
        self.objects.get(id)
    }

    /// Returns a mutable reference to the associated object and its position, using the handle.
    pub fn get_mut(&mut self, id: ShapeGridHandle) -> Option<&mut StoreObject<O, AB>> {
        self.objects.get_mut(id)
    }

    /// The underlying storage
    pub fn storage(&self) -> &SparseStorage<ShapeGridCell> {
        &self.storage
    }

    /// Queries for objects intersecting a given AABB.
    pub fn query(&self, aabb: AB) -> impl Iterator<Item = (ShapeGridHandle, &AB, &O)> + '_ {
        self.query_broad(aabb)
            .map(move |h| {
                let obj = &self.objects[h];
                (h, &obj.aabb, &obj.obj)
            })
            .filter(move |&(_, x, _)| aabb.intersects(x))
    }

    /// Queries for all objects in the cells intersecting the given AABB
    pub fn query_broad(&self, bbox: AB) -> impl Iterator<Item = ShapeGridHandle> + '_ {
        let storage = &self.storage;

        let ll_id = storage.cell_id(bbox.ll());
        let ur_id = storage.cell_id(bbox.ur());

        let iter = cell_range(ll_id, ur_id)
            .flat_map(move |id| storage.cell(id))
            .flat_map(|x| x.objs.iter().copied());

        if ll_id == ur_id {
            QueryIter::Simple(iter)
        } else {
            QueryIter::Dedup(
                fnv::FnvHashSet::with_capacity_and_hasher(5, fnv::FnvBuildHasher::default()),
                iter,
            )
        }
    }

    /// Returns the number of objects currently available
    /// (removals that were not confirmed with maintain() are still counted)
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Checks if the grid contains objects or not
    /// (removals that were not confirmed with maintain() are still counted)
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

fn cells_apply<AB: AABB>(
    storage: &mut SparseStorage<ShapeGridCell>,
    bbox: &AB,
    f: impl Fn(&mut ShapeGridCell, bool),
) {
    let ll = storage.cell_mut(bbox.ll()).0;
    let ur = storage.cell_mut(bbox.ur()).0;
    for id in cell_range(ll, ur) {
        f(storage.cell_mut_unchecked(id), ll == ur)
    }
}

enum QueryIter<T: Iterator<Item = (ShapeGridHandle, bool)>> {
    Simple(T),
    Dedup(fnv::FnvHashSet<ShapeGridHandle>, T),
}

impl<T: Iterator<Item = (ShapeGridHandle, bool)>> Iterator for QueryIter<T> {
    type Item = ShapeGridHandle;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            QueryIter::Simple(x) => x.next().map(|(x, _)| x),
            QueryIter::Dedup(seen, x) => {
                for (v, sing_cell) in x {
                    if sing_cell {
                        return Some(v);
                    }
                    if seen.insert(v) {
                        return Some(v);
                    }
                }
                None
            }
        }
    }
}
