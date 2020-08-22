use crate::{Lots, ProjectKind, Road, RoadID, Roads, SpatialMap};
use geom::obb::OBB;
use geom::segment::Segment;
use geom::Vec2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LotID;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Lot {
    pub id: LotID,
    pub parent: RoadID,
    pub shape: OBB,
    pub road_edge: Segment,
}

impl Lot {
    pub fn try_make(
        lots: &mut Lots,
        spatial: &mut SpatialMap,
        roads: &Roads,
        parent: RoadID,
        at: Vec2,
        axis: Vec2,
        size: f32,
    ) -> Option<LotID> {
        let shape = OBB::new(at + axis * size * 0.5, axis, size, size);

        for obj in spatial.query_rect(shape.bbox()) {
            match obj {
                ProjectKind::Road(r) => {
                    let r = &roads[r];
                    if r.intersects(shape) {
                        return None;
                    }
                }
                ProjectKind::Lot(h) => {
                    let h = &lots[h];
                    if h.shape.intersects(shape) {
                        return None;
                    }
                }
                _ => {}
            }
        }

        let road_edge = Segment::new(shape.corners[0], shape.corners[1]);

        let id = lots.insert_with_key(move |id| Lot {
            id,
            parent,
            shape,
            road_edge,
        });
        spatial.insert_lot(&lots[id]);
        Some(id)
    }
}
