use crate::geometry::gridstore::ObjectState::{NewPos, Removed, Unchanged};
use crate::geometry::rect::Rect;
use cgmath::{MetricSpace, Vector2};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridStoreHandle(usize);

#[derive(PartialEq, Eq)]
enum ObjectState {
    Unchanged,
    NewPos,
    Removed,
}

#[derive(Clone)]
pub struct CellObject {
    pub id: GridStoreHandle,
    pub pos: Vector2<f32>,
}

impl CellObject {
    pub fn new(id: GridStoreHandle, pos: Vector2<f32>) -> Self {
        Self { id, pos }
    }
}

struct StoreObject<O> {
    obj: O,
    state: ObjectState,
    pos: Vector2<f32>,
    cell_id: usize,
}

#[derive(Default)]
struct GridStoreCell {
    objs: Vec<CellObject>,
    dirty: bool,
}

pub struct GridStore<O> {
    start_x: i32,
    start_y: i32,
    cell_size: i32,
    width: u32,
    height: u32,
    cells: Vec<GridStoreCell>,
    objects: HashMap<GridStoreHandle, StoreObject<O>>, // FIXME: Optimize using a slab
    id: usize,
}

impl<O> GridStore<O> {
    pub fn new(cell_size: i32) -> Self {
        Self {
            start_x: -10 * cell_size,
            start_y: -10 * cell_size,
            cell_size,
            width: 20,
            height: 20,
            cells: (0..20 * 20).map(|_| GridStoreCell::default()).collect(),
            objects: HashMap::new(),
            id: 0,
        }
    }

    pub fn insert(&mut self, pos: Vector2<f32>, obj: O) -> GridStoreHandle {
        self.check_resize(pos);
        self.id += 1;
        let handle = GridStoreHandle(self.id);
        let cell_id = self.get_cell_id(pos);
        self.objects.insert(
            handle,
            StoreObject {
                obj,
                state: ObjectState::Unchanged,
                pos,
                cell_id,
            },
        );
        self.get_cell_mut(cell_id)
            .objs
            .push(CellObject::new(handle, pos));
        handle
    }

    pub fn set_position(&mut self, handle: GridStoreHandle, pos: Vector2<f32>) {
        let cell_id = self.get_cell_id(pos);
        self.get_cell_mut(cell_id).dirty = true;
        let obj = self
            .objects
            .get_mut(&handle)
            .expect("Object not in grid anymore");
        obj.cell_id = cell_id;
        obj.pos = pos;
        obj.state = ObjectState::NewPos;
    }

    pub fn remove(&mut self, handle: GridStoreHandle) {
        let st = self
            .objects
            .get_mut(&handle)
            .expect("Object not in grid anymore");
        match st.state {
            NewPos => {
                panic!("Cannot remove moved object");
            }
            Unchanged => {
                st.state = Removed;
                let p = st.pos;
                let cell_id = self.get_cell_id(p);
                self.get_cell_mut(cell_id).dirty = true;
            }
            Removed => {}
        }
    }

    pub fn maintain(&mut self) {
        let mut to_add = vec![];
        const DELETE_ID: usize = 100_000_000;

        for (id, cell) in self.cells.iter_mut().filter(|x| x.dirty).enumerate() {
            cell.dirty = false;

            for cellobj in cell.objs.iter_mut() {
                let store_obj = self.objects.get_mut(&cellobj.id).unwrap();
                match store_obj.state {
                    ObjectState::NewPos => {
                        cellobj.pos = store_obj.pos;
                        if store_obj.cell_id != id {
                            to_add.push((store_obj.cell_id, cellobj.clone()));
                            cellobj.id = GridStoreHandle(DELETE_ID);
                        }
                    }

                    ObjectState::Removed => {
                        cellobj.id = GridStoreHandle(DELETE_ID);
                    }
                    ObjectState::Unchanged => {}
                }
                store_obj.state = Unchanged;
            }

            cell.objs.retain(|x| x.id.0 != DELETE_ID);
        }

        for (cell_id, obj) in to_add {
            self.cells[cell_id].objs.push(obj);
        }
    }

    pub fn get_obj(&self, id: GridStoreHandle) -> &O {
        &self.objects.get(&id).unwrap().obj
    }

    #[rustfmt::skip]
    pub fn query_around(&self, pos: Vector2<f32>, radius: f32) -> Vec<&CellObject> {
        if radius > self.cell_size as f32 {
            println!(
                "asked radius ({}) bigger than cell_size ({}): might omit some results",
                radius, self.cell_size
            );
        }
        let mut objs = vec![];

        let radius2 = radius * radius;

        let cell = self.get_cell_id(pos) as i32;

        self.populate_objs(cell, pos, radius, radius2, &mut objs);
        self.populate_objs(cell + 1, pos, radius, radius2, &mut objs);
        self.populate_objs(cell - 1, pos, radius, radius2, &mut objs);
        self.populate_objs(cell + self.width as i32, pos, radius, radius2, &mut objs);
        self.populate_objs(cell - self.width as i32, pos, radius, radius2, &mut objs);
        self.populate_objs(cell + self.width as i32 + 1, pos, radius, radius2, &mut objs);
        self.populate_objs(cell + self.width as i32 - 1, pos, radius, radius2, &mut objs);
        self.populate_objs(cell - self.width as i32 - 1, pos, radius, radius2, &mut objs);
        self.populate_objs(cell - self.width as i32 - 1, pos, radius, radius2, &mut objs);

        objs
    }

    #[inline(always)]
    fn populate_objs<'a>(
        &'a self,
        cell_id: i32,
        pos: Vector2<f32>,
        radius: f32,
        radius2: f32,
        objs: &mut Vec<&'a CellObject>,
    ) {
        if cell_id < 0 || cell_id >= (self.width * self.height) as i32 {
            return;
        }
        let (x, y) = self.get_cell_box(cell_id);
        if Rect::new_i32(x, y, self.cell_size, self.cell_size).contains_within(pos, radius) {
            for x in &self.get_cell(cell_id as usize).objs {
                if x.pos.distance2(pos) < radius2 {
                    objs.push(x);
                }
            }
        }
    }

    fn check_resize(&mut self, pos: Vector2<f32>) {
        let mut reallocate = false;
        while (pos.x as i32) < self.start_x {
            self.start_x -= self.cell_size;
            self.width += 1;
            reallocate = true;
        }

        while (pos.y as i32) < self.start_y {
            self.start_y -= self.cell_size;
            self.height += 1;
            reallocate = true;
        }

        while (pos.x as i32) > self.start_x + self.width as i32 * self.cell_size {
            self.width += 1;
            reallocate = true;
        }

        while (pos.y as i32) > self.start_y + self.height as i32 * self.cell_size {
            self.height += 1;
            self.cells
                .resize_with((self.width * self.height) as usize, GridStoreCell::default);
        }

        if reallocate {
            println!(
                "Resizing coworld to x: {} y: {} w: {} h: {}",
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
                obj.state = Unchanged;

                self.cells
                    .get_mut(cell_id)
                    .unwrap()
                    .objs
                    .push(CellObject::new(*id, obj.pos));
            }
        }
    }

    fn get_cell_box(&self, id: i32) -> (i32, i32) {
        (
            self.start_x + (id % self.width as i32) * self.cell_size,
            self.start_y + (id / self.width as i32) * self.cell_size,
        )
    }

    fn get_cell(&self, id: usize) -> &GridStoreCell {
        self.cells.get(id).expect("get_cell error")
    }

    fn get_cell_mut(&mut self, id: usize) -> &mut GridStoreCell {
        self.cells.get_mut(id).expect("get_cell error")
    }

    fn get_cell_id(&self, pos: Vector2<f32>) -> usize {
        Self::get_cell_id_raw(
            self.width as i32,
            self.start_x,
            self.start_y,
            self.cell_size,
            pos,
        )
    }

    fn get_cell_id_raw(
        width: i32,
        start_x: i32,
        start_y: i32,
        cell_size: i32,
        pos: Vector2<f32>,
    ) -> usize {
        let i_x = (pos.x as i32 - start_x) / cell_size;
        let i_y = (pos.y as i32 - start_y) / cell_size;
        (i_y * width + i_x) as usize
    }
}
