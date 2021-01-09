use crate::cell::{CellObject, GridCell};
use crate::storage::{cell_range, CellIdx, SparseStorage, Storage};
use geom::Vec2;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, Key, SlotMap};

pub type GridObjects<O> = SlotMap<GridHandle, StoreObject<O>>;

new_key_type! {
    /// This handle is used to modify the associated object or to update its position.
    /// It is returned by the _insert_ method of a Grid.
    pub struct GridHandle;
}

/// State of an object, maintain() updates the internals of the grid and resets this to Unchanged
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ObjectState {
    Unchanged,
    NewPos(Vec2),
    Relocate(Vec2, CellIdx),
    Removed,
}

/// The actual object stored in the store
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct StoreObject<O> {
    /// User-defined object to be associated with a value
    obj: O,
    pub state: ObjectState,
    pub pos: Vec2,
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
/// A SlotMap is used for objects managing, adding a level of indirection between points and objects.
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
///
/// ## Examples
/// Here is a basic example that shows most of its capabilities:
/// ```rust
/// use flat_spatial::Grid;
/// use geom::Vec2;
///
/// let mut g: Grid<i32> = Grid::new(10); // Creates a new grid with a cell width of 10 with an integer as extra data
/// let a = g.insert(Vec2::ZERO, 0); // Inserts a new element with data: 0
///
/// {
///     let mut before = g.query_around(Vec2::ZERO, 5.0).map(|(id, _pos)| id); // Queries for objects around a given point
///     assert_eq!(before.next(), Some(a));
///     assert_eq!(g.get(a).unwrap().1, &0);
/// }
/// let b = g.insert(Vec2::ZERO, 1); // Inserts a new element, assigning a new unique and stable handle, with data: 1
///
/// g.remove(a); // Removes a value using the handle given by `insert`
///              // This won't have an effect until g.maintain() is called
///
/// g.maintain(); // Maintains the grid, which applies all removals and position updates (not needed for insertions)
///
/// assert_eq!(g.handles().collect::<Vec<_>>(), vec![b]); // We check that the "a" object has been removed
///
/// let after: Vec<_> = g.query_around(Vec2::ZERO, 5.0).map(|(id, _pos)| id).collect(); // And that b is query-able
/// assert_eq!(after, vec![b]);
///
/// assert_eq!(g.get(b).unwrap().1, &1); // We also check that b still has his data associated
/// assert_eq!(g.get(a), None); // But that a doesn't exist anymore
/// ```
#[derive(Clone, Deserialize, Serialize)]
pub struct Grid<O, ST: Storage<GridCell> = SparseStorage<GridCell>> {
    storage: ST,
    objects: GridObjects<O>,
    // Cache maintain vec to avoid allocating every time maintain is called
    to_relocate: Vec<CellObject>,
}

impl<ST: Storage<GridCell>, O: Copy> Grid<O, ST> {
    /// Creates an empty grid.   
    /// The cell size should be about the same magnitude as your queries size.
    pub fn new(cell_size: i32) -> Self {
        Self::with_storage(ST::new(cell_size))
    }

    /// Creates an empty grid.   
    /// The cell size should be about the same magnitude as your queries size.
    pub fn with_storage(st: ST) -> Self {
        Self {
            storage: st,
            objects: SlotMap::with_key(),
            to_relocate: vec![],
        }
    }

    /// Inserts a new object with a position and an associated object
    /// Returns the unique and stable handle to be used with get_obj
    pub fn insert(&mut self, pos: Vec2, obj: O) -> GridHandle {
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
    pub fn set_position(&mut self, handle: GridHandle, pos: impl Into<Vec2>) {
        let pos = pos.into();

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
    /// use geom::Vec2;
    /// let mut g: Grid<()> = Grid::new(10);
    /// let h = g.insert(Vec2::new(5.0, 3.0), ());
    /// g.remove(h);
    /// ```
    pub fn remove(&mut self, handle: GridHandle) -> Option<O> {
        let obj = self.objects.get_mut(handle)?;

        obj.state = ObjectState::Removed;
        self.storage.cell_mut_unchecked(obj.cell_id).dirty = true;

        Some(obj.obj)
    }

    /// Maintains the world, updating all the positions (and moving them to corresponding cells)
    /// and removing necessary objects and empty cells.
    /// Runs in linear time O(N) where N is the number of objects.
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// use geom::Vec2;
    /// let mut g: Grid<()> = Grid::new(10);
    /// let h = g.insert(Vec2::new(5.0, 3.0), ());
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
    pub fn objects(&self) -> impl Iterator<Item = &O> + '_ {
        self.objects.values().map(|x| &x.obj)
    }

    /// Returns a reference to the associated object and its position, using the handle.  
    ///
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// use geom::Vec2;
    /// let mut g: Grid<i32> = Grid::new(10);
    /// let h = g.insert(Vec2::new(5.0, 3.0), 42);
    /// assert_eq!(g.get(h), Some((Vec2::new(5.0, 3.0).into(), &42)));
    /// ```
    pub fn get(&self, id: GridHandle) -> Option<(Vec2, &O)> {
        self.objects.get(id).map(|x| (x.pos, &x.obj))
    }

    /// Returns a mutable reference to the associated object and its position, using the handle.  
    ///
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// use geom::Vec2;
    /// let mut g: Grid<i32> = Grid::new(10);
    /// let h = g.insert(Vec2::new(5.0, 3.0), 42);
    /// *g.get_mut(h).unwrap().1 = 56;
    /// assert_eq!(g.get(h).unwrap().1, &56);
    /// ```    
    pub fn get_mut(&mut self, id: GridHandle) -> Option<(Vec2, &mut O)> {
        self.objects.get_mut(id).map(|x| (x.pos, &mut x.obj))
    }

    /// The underlying storage
    pub fn storage(&self) -> &ST {
        &self.storage
    }

    pub fn query_around(&self, pos: Vec2, radius: f32) -> impl Iterator<Item = CellObject> + '_ {
        let ll = pos - Vec2::splat(radius);
        let ur = pos + Vec2::splat(radius);

        let radius2 = radius * radius;
        self.query_raw(ll, ur).filter(move |(_, pos_obj)| {
            let x = pos_obj.x - pos.x;
            let y = pos_obj.y - pos.y;
            x * x + y * y < radius2
        })
    }

    pub fn query_aabb(&self, ll_: Vec2, ur_: Vec2) -> impl Iterator<Item = CellObject> + '_ {
        let ll = ll_.min(ur_);
        let ur = ll_.max(ur_);

        self.query_raw(ll, ur).filter(move |(_, pos_obj)| {
            (ll.x..=ur.x).contains(&pos_obj.x) && (ll.y..=ur.y).contains(&pos_obj.y)
        })
    }

    /// Queries for all objects in the cells intersecting an axis-aligned rectangle defined by lower left (ll) and upper right (ur)
    /// Try to keep the rect's width/height of similar magnitudes to the cell size for better performance.
    ///
    /// # Example
    /// ```rust
    /// use flat_spatial::Grid;
    /// use geom::Vec2;
    ///
    /// let mut g: Grid<()> = Grid::new(10);
    /// let a = g.insert(Vec2::ZERO, ());
    /// let b = g.insert(Vec2::new(5.0, 5.0), ());
    ///
    /// let around: Vec<_> = g.query_raw([-1.0, -1.0].into(), [1.0, 1.0].into()).map(|(id, _pos)| id).collect();
    ///
    /// assert_eq!(vec![a, b], around);
    /// ```
    pub fn query_raw(&self, ll: Vec2, ur: Vec2) -> impl Iterator<Item = CellObject> + '_ {
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
