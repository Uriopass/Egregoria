use crate::cell::ShapeGridCell;
use crate::storage::{cell_range, SparseStorage, Storage};
use common::FastSet;
use geom::{Circle, Intersect, Shape, Vec2, AABB};
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

pub type ShapeGridObjects<O, S> = SlotMap<ShapeGridHandle, StoreObject<O, S>>;

new_key_type! {
    /// This handle is used to modify the associated object or to update its position.
    /// It is returned by the _insert_ method of a ShapeGrid.
    pub struct ShapeGridHandle;
}

/// The actual object stored in the store
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct StoreObject<O: Copy, S: Shape> {
    /// User-defined object to be associated with a value
    obj: O,
    pub shape: S,
}

/// ShapeGrid is a generic shape-based spatial partitioning structure that uses a generic storage of cells which acts as a
/// grid instead of a tree.
///
/// ## Fast queries
/// In theory, ShapeGrid should be faster than a quadtree/r-tree because it has no log costs
/// (calculating the cells around a point is trivial).  
/// However, it only works if the cell size is adapted to the problem, much like how a tree has to
/// be balanced to be efficient.  
///
/// ## Dynamicity
/// ShapeGrid's allows eager removals and position updates, however for big shapes (spanning many cells)
/// this can be expensive, so beware.
///
/// Use this grid for mostly static objects with the occasional removal/position update if needed.
///
/// A SlotMap is used for objects managing, adding a level of indirection between shapes and objects.
/// SlotMap is used because removal doesn't alter handles given to the user, while still having constant time access.
/// However it requires O to be copy, but SlotMap's author stated that they were working on a similar
/// map where Copy isn't required.
///
/// ## About object managment
///
/// In theory, you don't have to use the object managment directly, you can make your custom
/// Handle -> Object map by specifying "`()`" to be the object type.
/// _(This can be useful if your object is not Copy)_
/// Since `()` is zero sized, it should probably optimize away a lot of the object managment code.
///
/// ```rust
/// use flat_spatial::ShapeGrid;
/// use geom::Circle;
///
/// let mut g: ShapeGrid<(), Circle> = ShapeGrid::new(10);
/// let handle = g.insert(Circle {
///     center: [0.0, 0.0].into(),
///     radius: 3.0,
/// }, ());
/// // Use handle however you want
/// ```
#[derive(Clone, Deserialize, Serialize)]
pub struct ShapeGrid<
    O: Copy,
    S: Shape + Intersect<AABB> + Copy,
    ST: Storage<ShapeGridCell> = SparseStorage<ShapeGridCell>,
> {
    storage: ST,
    objects: ShapeGridObjects<O, S>,
}

impl<S: Shape + Intersect<AABB> + Copy, ST: Storage<ShapeGridCell>, O: Copy> ShapeGrid<O, S, ST> {
    /// Creates an empty grid.
    /// The cell size should be about the same magnitude as your queries size.
    pub fn new(cell_size: i32) -> Self {
        Self {
            storage: ST::new(cell_size),
            objects: ShapeGridObjects::default(),
        }
    }

    /// Clears the grid.
    pub fn clear(&mut self) -> impl Iterator<Item = (S, O)> {
        self.storage = ST::new(self.storage.cell_size());
        let objs = std::mem::take(&mut self.objects);
        objs.into_iter().map(|(_, o)| (o.shape, o.obj))
    }

    /// Creates an empty grid.   
    /// The cell size should be about the same magnitude as your queries size.
    pub fn with_storage(st: ST) -> Self {
        Self {
            storage: st,
            objects: ShapeGridObjects::default(),
        }
    }

    fn cells_apply(storage: &mut ST, shape: &S, f: impl Fn(&mut ShapeGridCell, bool)) {
        let bbox = shape.bbox();
        let ll = storage.cell_mut(bbox.ll).0;
        let ur = storage.cell_mut(bbox.ur).0;
        for id in cell_range(ll, ur) {
            if !shape.intersects(&storage.cell_aabb(id)) {
                continue;
            }
            f(storage.cell_mut_unchecked(id), ll == ur)
        }
    }

    /// Inserts a new object with a position and an associated object
    /// Returns the unique and stable handle to be used with get_obj
    pub fn insert(&mut self, shape: S, obj: O) -> ShapeGridHandle {
        let Self {
            storage, objects, ..
        } = self;

        let h = objects.insert(StoreObject { obj, shape });
        Self::cells_apply(storage, &shape, |cell, sing_cell| {
            cell.objs.push((h, sing_cell));
        });
        h
    }

    /// Updates the shape of an object.
    pub fn set_shape(&mut self, handle: ShapeGridHandle, shape: S) {
        let obj = self
            .objects
            .get_mut(handle)
            .expect("Object not in grid anymore");

        let storage = &mut self.storage;

        Self::cells_apply(storage, &obj.shape, |cell, _| {
            let p = match cell.objs.iter().position(|(x, _)| *x == handle) {
                Some(x) => x,
                None => return,
            };
            cell.objs.swap_remove(p);
        });

        Self::cells_apply(storage, &shape, |cell, sing_cell| {
            cell.objs.push((handle, sing_cell))
        });

        obj.shape = shape;
    }

    /// Removes an object from the grid.
    pub fn remove(&mut self, handle: ShapeGridHandle) -> Option<O> {
        let st = self.objects.remove(handle)?;

        let storage = &mut self.storage;
        Self::cells_apply(storage, &st.shape, |cell, _| {
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
    pub fn get(&self, id: ShapeGridHandle) -> Option<(&S, &O)> {
        self.objects.get(id).map(|x| (&x.shape, &x.obj))
    }

    /// Returns a mutable reference to the associated object and its position, using the handle.
    pub fn get_mut(&mut self, id: ShapeGridHandle) -> Option<(&S, &mut O)> {
        self.objects.get_mut(id).map(|x| (&x.shape, &mut x.obj))
    }

    /// The underlying storage
    pub fn storage(&self) -> &ST {
        &self.storage
    }

    /// Queries for objects intersecting a given shape.
    pub fn query<'a, QS: 'a + Shape + Intersect<AABB> + Intersect<S> + Clone>(
        &'a self,
        shape: QS,
    ) -> impl Iterator<Item = (ShapeGridHandle, &S, &O)> + 'a {
        self.query_broad(shape.clone())
            .map(move |h| {
                let obj = &self.objects[h];
                (h, &obj.shape, &obj.obj)
            })
            .filter(move |&(_, x, _)| shape.intersects(x))
    }

    /// Queries for all objects in the cells intersecting the given shape
    pub fn query_broad<'a, QS: 'a + Shape + Intersect<AABB>>(
        &'a self,
        shape: QS,
    ) -> impl Iterator<Item = ShapeGridHandle> + 'a {
        let bbox = shape.bbox();
        let storage = &self.storage;

        let ll_id = storage.cell_id(bbox.ll);
        let ur_id = storage.cell_id(bbox.ur);

        let iter = cell_range(ll_id, ur_id)
            .filter(move |&id| shape.intersects(&storage.cell_aabb(id)))
            .flat_map(move |id| storage.cell(id))
            .flat_map(|x| x.objs.iter().copied());

        if ll_id == ur_id {
            QueryIter::Simple(iter)
        } else {
            QueryIter::Dedup(common::fastset_with_capacity(5), iter)
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

impl<S: Shape + Intersect<AABB> + Copy, ST: Storage<ShapeGridCell>, O: Copy> ShapeGrid<O, S, ST>
where
    Circle: Intersect<S>,
{
    /// Queries for objects around a point, same as querying a circle at pos with a given radius.
    pub fn query_around(
        &self,
        pos: Vec2,
        radius: f32,
    ) -> impl Iterator<Item = (ShapeGridHandle, &S, &O)> + '_ {
        self.query(Circle {
            center: pos,
            radius,
        })
    }
}

enum QueryIter<T: Iterator<Item = (ShapeGridHandle, bool)>> {
    Simple(T),
    Dedup(FastSet<ShapeGridHandle>, T),
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
