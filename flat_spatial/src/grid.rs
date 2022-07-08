use crate::cell::{CellObject, GridCell};
use crate::storage::{cell_range, CellIdx, SparseStorage};
use crate::Vec2;
use slotmap::{new_key_type, SlotMap};
use std::marker::PhantomData;

pub type GridObjects<O, V2> = SlotMap<GridHandle, StoreObject<O, V2>>;

new_key_type! {
    /// This handle is used to modify the associated object or to update its position.
    /// It is returned by the _insert_ method of a Grid.
    pub struct GridHandle;
}

/// State of an object, maintain() updates the internals of the grid and resets this to Unchanged
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ObjectState<V2: Vec2> {
    Unchanged,
    NewPos(V2),
    Relocate(V2, CellIdx),
    Removed,
}

/// The actual object stored in the store
#[derive(Clone, Copy)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StoreObject<O, V2: Vec2> {
    /// User-defined object to be associated with a value
    obj: O,
    pub state: ObjectState<V2>,
    pub pos: V2,
    pub cell_id: CellIdx,
}

/// Grid is a point-based spatial partitioning structure that uses a generic storage of cells which acts as a
/// grid instead of a tree.
///
/// ## Fast queries
/// In theory, Grid should be faster than a quadtree/r-tree because it has no log costs
/// (calculating the cells around a point is trivial).  
/// However, it only works if the cell size is adapted to the problem, much like how a tree has to
/// be balanced to be efficient.  
///
/// ## Dynamicity
/// Grid's big advantage is that it is dynamic, supporting lazy positions updates
/// and object removal in constant time. Once objects are in, there is almost no allocation happening.
///
/// Compare that to most immutable spatial partitioning structures out there, which pretty much require
/// to rebuild the entire tree every time.
///
/// A `SlotMap` is used for objects managing, adding a level of indirection between points and objects.
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
///
/// ## Examples
/// Here is a basic example that shows most of its capabilities:
/// ```rust
/// use flat_spatial::Grid;
///
/// let mut g: Grid<i32, [f32; 2]> = Grid::new(10); // Creates a new grid with a cell width of 10 with an integer as extra data
/// let a = g.insert([0.0, 0.0], 0); // Inserts a new element with data: 0
///
/// {
///     let mut before = g.query_around([0.0, 0.0], 5.0).map(|(id, _pos)| id); // Queries for objects around a given point
///     assert_eq!(before.next(), Some(a));
///     assert_eq!(g.get(a).unwrap().1, &0);
/// }
/// let b = g.insert([0.0, 0.0], 1); // Inserts a new element, assigning a new unique and stable handle, with data: 1
///
/// g.remove(a); // Removes a value using the handle given by `insert`
///              // This won't have an effect until g.maintain() is called
///
/// g.maintain(); // Maintains the grid, which applies all removals and position updates (not needed for insertions)
///
/// assert_eq!(g.handles().collect::<Vec<_>>(), vec![b]); // We check that the "a" object has been removed
///
/// let after: Vec<_> = g.query_around([0.0, 0.0], 5.0).map(|(id, _pos)| id).collect(); // And that b is query-able
/// assert_eq!(after, vec![b]);
///
/// assert_eq!(g.get(b).unwrap().1, &1); // We also check that b still has his data associated
/// assert_eq!(g.get(a), None); // But that a doesn't exist anymore
/// ```
#[derive(Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Grid<O, V2: Vec2> {
    storage: SparseStorage<GridCell<V2>>,
    objects: GridObjects<O, V2>,
    // Cache maintain vec to avoid allocating every time maintain is called
    to_relocate: Vec<CellObject<V2>>,
    _phantom: PhantomData<V2>,
}

impl<O: Copy, V2: Vec2> Grid<O, V2> {
    /// Creates an empty grid.   
    /// The cell size should be about the same magnitude as your queries size.
    pub fn new(cell_size: i32) -> Self {
        Self {
            storage: SparseStorage::new(cell_size),
            objects: SlotMap::with_key(),
            to_relocate: vec![],
            _phantom: Default::default(),
        }
    }

    /// Inserts a new object with a position and an associated object
    /// Returns the unique and stable handle to be used with `get_obj`
    pub fn insert(&mut self, pos: V2, obj: O) -> GridHandle {
        let (cell_id, cell) = self.storage.cell_mut(pos);
        let handle = self.objects.insert(StoreObject {
            obj,
            state: ObjectState::Unchanged,
            pos,
            cell_id,
        });
        cell.objs.push((handle, pos));
        handle
    }

    /// Lazily sets the position of an object (if it is not marked for deletion).
    /// This won't be taken into account until maintain() is called.
    pub fn set_position(&mut self, handle: GridHandle, pos: V2) {
        let obj = match self.objects.get_mut(handle) {
            Some(x) => x,
            None => {
                debug_assert!(false, "Object not in grid anymore");
                return;
            }
        };

        if matches!(obj.state, ObjectState::Removed) {
            return;
        }

        let target_id = self.storage.cell_id(pos);
        obj.state = if target_id == obj.cell_id {
            ObjectState::NewPos(pos)
        } else {
            ObjectState::Relocate(pos, target_id)
        };

        self.storage.cell_mut_unchecked(obj.cell_id).dirty = true;
    }

    /// Lazily removes an object from the grid.
    /// This won't be taken into account until maintain() is called.  
    ///
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// let mut g: Grid<(), [f32; 2]> = Grid::new(10);
    /// let h = g.insert([5.0, 3.0], ());
    /// g.remove(h);
    /// ```
    pub fn remove(&mut self, handle: GridHandle) -> Option<O> {
        let obj = self.objects.get_mut(handle)?;

        obj.state = ObjectState::Removed;
        self.storage.cell_mut_unchecked(obj.cell_id).dirty = true;

        Some(obj.obj)
    }

    /// Directly removes an object from the grid.
    /// This is equivalent to remove() then maintain() but is much faster (O(1))
    ///
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// let mut g: Grid<(), [f32; 2]> = Grid::new(10);
    /// let h = g.insert([5.0, 3.0], ());
    /// g.remove(h);
    /// ```
    pub fn remove_maintain(&mut self, handle: GridHandle) -> Option<O> {
        let obj = self.objects.remove(handle)?;

        let cell = self.storage.cell_mut_unchecked(obj.cell_id);

        cell.objs.retain(|&(h, _)| h != handle);

        Some(obj.obj)
    }

    /// Clear all objects from the grid.
    /// Returns the objects and their positions.
    pub fn clear(&mut self) -> impl Iterator<Item = (V2, O)> {
        let objects = std::mem::take(&mut self.objects);
        self.storage = SparseStorage::new(self.storage.cell_size());
        self.to_relocate.clear();
        objects.into_iter().map(|(_, x)| (x.pos, x.obj))
    }

    /// Maintains the world, updating all the positions (and moving them to corresponding cells)
    /// and removing necessary objects and empty cells.
    /// Runs in linear time O(N) where N is the number of objects.
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// let mut g: Grid<(), [f32; 2]> = Grid::new(10);
    /// let h = g.insert([5.0, 3.0], ());
    /// g.remove(h);
    ///
    /// assert!(g.get(h).is_some());
    /// g.maintain();
    /// assert!(g.get(h).is_none());
    /// ```
    pub fn maintain(&mut self) {
        let Self {
            storage,
            objects,
            to_relocate,
            ..
        } = self;

        storage.modify(|cell| {
            cell.maintain(objects, to_relocate);
            cell.objs.is_empty()
        });

        for (handle, pos) in to_relocate.drain(..) {
            storage.cell_mut(pos).1.objs.push((handle, pos));
        }
    }

    /// Iterate over all handles
    pub fn handles(&self) -> impl Iterator<Item = GridHandle> + '_ {
        self.objects.keys()
    }

    /// Iterate over all objects
    pub fn objects(&self) -> impl Iterator<Item = (V2, &O)> + '_ {
        self.objects.values().map(|x| (x.pos, &x.obj))
    }

    /// Returns a reference to the associated object and its position, using the handle.  
    ///
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// let mut g: Grid<i32, [f32; 2]> = Grid::new(10);
    /// let h = g.insert([5.0, 3.0], 42);
    /// assert_eq!(g.get(h), Some(([5.0, 3.0], &42)));
    /// ```
    pub fn get(&self, id: GridHandle) -> Option<(V2, &O)> {
        self.objects.get(id).map(|x| (x.pos, &x.obj))
    }

    /// Returns a mutable reference to the associated object and its position, using the handle.  
    ///
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// let mut g: Grid<i32, [f32; 2]> = Grid::new(10);
    /// let h = g.insert([5.0, 3.0], 42);
    /// *g.get_mut(h).unwrap().1 = 56;
    /// assert_eq!(g.get(h).unwrap().1, &56);
    /// ```    
    pub fn get_mut(&mut self, id: GridHandle) -> Option<(V2, &mut O)> {
        self.objects.get_mut(id).map(|x| (x.pos, &mut x.obj))
    }

    /// The underlying storage
    pub fn storage(&self) -> &SparseStorage<GridCell<V2>> {
        &self.storage
    }

    pub fn query_around(&self, pos: V2, radius: f32) -> impl Iterator<Item = CellObject<V2>> + '_ {
        let ll = [pos.x() - radius, pos.y() - radius];
        let ur = [pos.x() + radius, pos.y() + radius];

        let radius2 = radius * radius;
        self.query(ll.into(), ur.into())
            .filter(move |(_, pos_obj)| {
                let x = pos_obj.x() - pos.x();
                let y = pos_obj.y() - pos.y();
                x * x + y * y < radius2
            })
    }

    pub fn query_aabb(&self, ll_: V2, ur_: V2) -> impl Iterator<Item = CellObject<V2>> + '_ {
        let ll = [ll_.x().min(ur_.x()), ll_.y().min(ur_.y())];
        let ur = [ll_.x().max(ur_.x()), ll_.y().max(ur_.y())];

        self.query(ll.into(), ur.into())
            .filter(move |(_, pos_obj)| {
                (ll[0]..=ur[0]).contains(&pos_obj.x()) && (ll[1]..=ur[1]).contains(&pos_obj.y())
            })
    }

    /// Queries for all objects in the cells intersecting an axis-aligned rectangle defined by lower left (ll) and upper right (ur)
    /// Try to keep the rect's width/height of similar magnitudes to the cell size for better performance.
    ///
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    ///
    /// let mut g: Grid<(), [f32; 2]> = Grid::new(10);
    /// let a = g.insert([0.0, 0.0], ());
    /// let b = g.insert([5.0, 5.0], ());
    ///
    /// let around: Vec<_> = g.query([-1.0, -1.0].into(), [1.0, 1.0].into()).map(|(id, _pos)| id).collect();
    ///
    /// assert_eq!(vec![a, b], around);
    /// ```
    pub fn query(&self, ll: V2, ur: V2) -> impl Iterator<Item = CellObject<V2>> + '_ {
        let ll_id = self.storage.cell_id(ll);
        let ur_id = self.storage.cell_id(ur);

        cell_range(ll_id, ur_id)
            .flat_map(move |id| self.storage.cell(id))
            .flat_map(|x| x.objs.iter().copied())
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
