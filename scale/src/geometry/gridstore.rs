use super::Vec2;
use cgmath::{Array, InnerSpace};
use slotmap::new_key_type;
use slotmap::SlotMap;

new_key_type! {
    /// This handle is used to modify the store object or to update the position
    pub struct GridStoreHandle;
}

/// State of an object, maintain() updates the internals of the gridstore and resets this to Unchanged
#[derive(Clone, Copy, PartialEq, Eq)]
enum ObjectState {
    Unchanged,
    NewPos,
    Removed,
}

/// The object stored in cells
#[derive(Clone)]
pub struct CellObject {
    pub id: GridStoreHandle,
    pub pos: Vec2,
}

impl CellObject {
    pub fn new(id: GridStoreHandle, pos: Vec2) -> Self {
        Self { id, pos }
    }
}

/// The actual object stored in the store
#[derive(Clone, Copy)]
struct StoreObject<O: Copy> {
    /// User-defined object to be associated with a value
    obj: O,
    state: ObjectState,
    pos: Vec2,
    cell_id: usize,
}

/// A single cell of the store, can be empty
#[derive(Default)]
pub struct GridStoreCell {
    pub objs: Vec<CellObject>,
    pub dirty: bool,
}

/// A gridstore contains object in a dense array of cells that are lists
/// The objective is to be able to perform range queries very fast without the log cost of a quadtree.
/// The gridstore is dynamic and will be resized if an object is added outside its range.
/// All methods except query_around and maintain are O(1)
///
/// A SlotMap is used for objects so that removal doesn't alter handles given to the user, while still having constant time access.
/// However it requires O to be copy, but SlotMap's author stated that he was working on a similar map where Copy isn't required.
pub struct GridStore<O: Copy> {
    start_x: i32,
    start_y: i32,
    cell_size: i32,
    width: i32,
    height: i32,
    cells: Vec<GridStoreCell>,
    objects: SlotMap<GridStoreHandle, StoreObject<O>>,
}

impl<O: Copy> GridStore<O> {
    /// Creates a new store centered on zero with width 20 and height 20
    pub fn new(cell_size: i32) -> Self {
        Self {
            start_x: -10 * cell_size,
            start_y: -10 * cell_size,
            cell_size,
            width: 20,
            height: 20,
            cells: (0..20 * 20).map(|_| GridStoreCell::default()).collect(),
            objects: SlotMap::with_key(),
        }
    }

    /// Inserts a new object with a position and an associated object
    /// Returns the handle
    pub fn insert(&mut self, pos: Vec2, obj: O) -> GridStoreHandle {
        self.check_resize(pos);
        let cell_id = self.get_cell_id(pos);
        let handle = self.objects.insert(StoreObject {
            obj,
            state: ObjectState::Unchanged,
            pos,
            cell_id,
        });
        self.get_cell_mut(cell_id)
            .objs
            .push(CellObject::new(handle, pos));
        handle
    }

    /// Sets the position of an object. Note that this won't be taken into account until maintain() is called
    pub fn set_position(&mut self, handle: GridStoreHandle, pos: Vec2) {
        self.check_resize(pos);
        let new_cell_id = self.get_cell_id(pos);
        let obj = self
            .objects
            .get_mut(handle)
            .expect("Object not in grid anymore");
        let old_id = obj.cell_id;
        obj.cell_id = new_cell_id;
        obj.pos = pos;
        match obj.state {
            ObjectState::Removed => {}
            _ => obj.state = ObjectState::NewPos,
        }

        self.get_cell_mut(old_id).dirty = true;
    }

    /// Removes an object from the store. Note that this won't be taken into account until maintain() is called
    pub fn remove(&mut self, handle: GridStoreHandle) {
        let st = self
            .objects
            .get_mut(handle)
            .expect("Object not in grid anymore");

        st.state = ObjectState::Removed;
        let id = st.cell_id;
        self.get_cell_mut(id).dirty = true;
    }

    /// Maintains the world, updating all the positions (and moving them to corresponding cells) and removing necessary objects.
    pub fn maintain(&mut self) {
        let mut to_add = vec![];

        for (id, cell) in self.cells.iter_mut().filter(|x| x.dirty).enumerate() {
            cell.dirty = false;

            for cellobj in cell.objs.iter_mut() {
                let store_obj = self.objects.get_mut(cellobj.id).unwrap();
                match store_obj.state {
                    ObjectState::NewPos => {
                        cellobj.pos = store_obj.pos;
                        if store_obj.cell_id != id {
                            to_add.push((store_obj.cell_id, cellobj.clone()));
                            cellobj.pos.x = std::f32::INFINITY; // Mark object for deletion
                        }
                        store_obj.state = ObjectState::Unchanged;
                    }
                    ObjectState::Removed => {
                        cellobj.pos.x = std::f32::INFINITY; // Mark object for deletion from cell
                        self.objects.remove(cellobj.id);
                    }
                    _ => {}
                }
            }

            cell.objs.retain(|x| x.pos.x.is_finite());
        }

        for (cell_id, obj) in to_add {
            self.cells[cell_id].objs.push(obj);
        }
    }

    pub fn get_obj(&self, id: GridStoreHandle) -> &O {
        &self.objects[id].obj
    }

    pub fn get_obj_mut(&mut self, id: GridStoreHandle) -> &mut O {
        &mut self.objects.get_mut(id).unwrap().obj
    }

    /// Queries for all objects around a position within a certain radius.
    /// Note that if the radius is bigger than the cell size, query_around might omit some results
    #[rustfmt::skip]
    pub fn query_around(&self, pos: Vec2, radius: f32) -> impl Iterator<Item = &CellObject> {

        let cell = self.get_cell_id(pos);
        let mut objs: Vec<&GridStoreCell> = Vec::with_capacity(4);

        objs.push(&self.cells[cell as usize]);

        let cell = cell as i32;

        let (w, h) = (self.width as i32, self.height as i32);
        let (x, y) = (cell % w, cell / w);

        let left:   bool = x > 0   && ((pos.x - radius) as i32) < self.start_x + x     * self.cell_size;
        let bottom: bool = y > 0   && ((pos.y - radius) as i32) < self.start_y + y     * self.cell_size;

        let right:  bool = x < w-1 && ((pos.x + radius) as i32) > self.start_x + (x+1) * self.cell_size;
        let top:    bool = y < h-1 && ((pos.y + radius) as i32) > self.start_y + (y+1) * self.cell_size;

        if right {
            self.populate_objs(cell + 1, &mut objs);
            if top {
                self.populate_objs(cell + w + 1, &mut objs);
            }
            if bottom {
                self.populate_objs(cell - w + 1, &mut objs);
            }
        }

        if left {
            self.populate_objs(cell - 1, &mut objs);
            if top {
                self.populate_objs(cell + w - 1, &mut objs);
            }
            if bottom {
                self.populate_objs(cell - w - 1, &mut objs);
            }
        }

        if top {
            self.populate_objs(cell + w, &mut objs);
        }

        if bottom {
            self.populate_objs(cell - w, &mut objs);
        }

        let radius2 = radius*radius;
        objs.into_iter().map(move |x| {
            x.objs.iter().filter(move |x| {
                (x.pos - pos).magnitude2() < radius2
            })
        }).flatten()
    }

    #[inline(always)]
    fn populate_objs<'a>(&'a self, cell_id: i32, objs: &mut Vec<&'a GridStoreCell>) {
        objs.push(&self.get_cell(cell_id as usize));
    }

    fn check_resize(&mut self, pos: Vec2) {
        assert!(pos.is_finite());

        let mut reallocate = false;

        let x = pos.x as i32;
        let y = pos.y as i32;

        if x <= self.start_x {
            let diff = 1 + (self.start_x - x) / self.cell_size;
            self.start_x -= self.cell_size * diff;
            self.width += diff;
            reallocate = true;
        }

        if y <= self.start_y {
            let diff = 1 + (self.start_y - y) / self.cell_size;
            self.start_y -= self.cell_size * diff;
            self.height += diff;
            reallocate = true;
        }

        let right = self.start_x + self.width as i32 * self.cell_size;
        if x >= right {
            self.width += 1 + (x - right) / self.cell_size;
            reallocate = true;
        }

        let up = self.start_y + self.height as i32 * self.cell_size;
        if y >= up {
            self.height += 1 + (y - up) / self.cell_size;
            self.cells
                .resize_with((self.width * self.height) as usize, GridStoreCell::default);
        }

        if reallocate {
            self.reallocate();
        }
    }

    fn reallocate(&mut self) {
        println!(
            "Reallocating coworld to x: {} y: {} w: {} h: {}",
            self.start_x,
            self.start_y,
            self.width as i32 * self.cell_size,
            self.height as i32 * self.cell_size
        );
        self.cells
            .resize_with((self.width * self.height) as usize, GridStoreCell::default);

        for x in &mut self.cells {
            x.objs.clear();
            x.dirty = false;
        }

        for (id, obj) in &mut self.objects {
            let cell_id = Self::get_cell_id_raw(
                self.width as i32,
                self.start_x,
                self.start_y,
                self.cell_size,
                obj.pos,
            );
            obj.cell_id = cell_id;
            obj.state = ObjectState::Unchanged;

            self.cells
                .get_mut(cell_id)
                .unwrap()
                .objs
                .push(CellObject::new(id, obj.pos));
        }
    }

    /// Get read access to the cells
    pub fn cells(&self) -> &Vec<GridStoreCell> {
        &self.cells
    }

    fn get_cell(&self, id: usize) -> &GridStoreCell {
        self.cells.get(id).expect("get_cell error")
    }

    fn get_cell_mut(&mut self, id: usize) -> &mut GridStoreCell {
        self.cells.get_mut(id).expect("get_cell error")
    }

    fn get_cell_id(&self, pos: Vec2) -> usize {
        Self::get_cell_id_raw(
            self.width as i32,
            self.start_x,
            self.start_y,
            self.cell_size,
            pos,
        )
    }

    fn get_cell_id_raw(width: i32, start_x: i32, start_y: i32, cell_size: i32, pos: Vec2) -> usize {
        let i_x = (pos.x as i32 - start_x) / cell_size;
        let i_y = (pos.y as i32 - start_y) / cell_size;
        (i_y * width + i_x) as usize
    }
}
