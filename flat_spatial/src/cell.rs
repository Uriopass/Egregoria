use crate::grid::{GridHandle, GridObjects, ObjectState};
use crate::shapegrid::ShapeGridHandle;
use geom::Vec2;
use serde::{Deserialize, Serialize};

pub type CellObject = (GridHandle, Vec2);

/// A single cell of the grid, can be empty
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct GridCell {
    pub objs: Vec<CellObject>,
    pub dirty: bool,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ShapeGridCell {
    pub objs: Vec<(ShapeGridHandle, bool)>,
}

impl GridCell {
    pub fn maintain<T: Copy>(
        &mut self,
        objects: &mut GridObjects<T>,
        to_relocate: &mut Vec<CellObject>,
    ) {
        if !self.dirty {
            return;
        }
        self.dirty = false;

        let mut i = 0;
        while i < self.objs.len() {
            let (obj_id, obj_pos) = unsafe { self.objs.get_unchecked_mut(i) };

            let store_obj = &mut objects[*obj_id];

            match store_obj.state {
                ObjectState::NewPos(pos) => {
                    store_obj.state = ObjectState::Unchanged;
                    store_obj.pos = pos;
                    *obj_pos = pos;
                    i += 1
                }
                ObjectState::Relocate(pos, target_id) => {
                    store_obj.state = ObjectState::Unchanged;
                    store_obj.pos = pos;
                    store_obj.cell_id = target_id;
                    to_relocate.push((*obj_id, pos));
                    self.objs.swap_remove(i);
                }
                ObjectState::Removed => {
                    objects.remove(*obj_id);
                    self.objs.swap_remove(i);
                }
                ObjectState::Unchanged => i += 1,
            }
        }
    }
}
