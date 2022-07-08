use crate::grid::{GridHandle, GridObjects, ObjectState};
use crate::shapegrid::ShapeGridHandle;
use crate::Vec2;

pub type CellObject<V2> = (GridHandle, V2);

/// A single cell of the grid, can be empty
#[derive(Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridCell<V2: Vec2> {
    pub objs: Vec<CellObject<V2>>,
    pub dirty: bool,
}

impl<V2: Vec2> Default for GridCell<V2> {
    fn default() -> Self {
        Self {
            objs: Vec::new(),
            dirty: false,
        }
    }
}

#[derive(Default, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShapeGridCell {
    pub objs: Vec<(ShapeGridHandle, bool)>,
}

impl<V2: Vec2> GridCell<V2> {
    pub(crate) fn maintain<T: Copy>(
        &mut self,
        objects: &mut GridObjects<T, V2>,
        to_relocate: &mut Vec<CellObject<V2>>,
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
