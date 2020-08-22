use crate::{Map, ProjectKind, RoadID};
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
        map: &mut Map,
        parent: RoadID,
        at: Vec2,
        axis: Vec2,
        size: f32,
    ) -> Option<LotID> {
        let shape = OBB::new(at + axis * size * 0.5, axis, size, size);

        for obj in map.spatial_map.query_rect(shape.bbox()) {
            match obj {
                ProjectKind::Road(r) => {
                    let r = &map.roads[r];
                    if r.project(shape.center()).distance(shape.center()) - r.width * 0.5
                        < size * 0.5
                    {
                        return None;
                    }
                }
                ProjectKind::Lot(h) => {
                    let h = &map.lots[h];
                    if h.shape.intersects(shape) {
                        return None;
                    }
                }
                _ => {}
            }
        }

        let road_edge = Segment::new(shape.corners[0], shape.corners[1]);

        let id = map.lots.insert_with_key(move |id| Lot {
            id,
            parent,
            shape,
            road_edge,
        });
        map.spatial_map.insert_lot(&map.lots[id]);
        Some(id)
    }
}
