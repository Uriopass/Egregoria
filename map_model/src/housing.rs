use crate::{Map, ProjectKind};
use geom::polyline::PolyLine;
use geom::Vec2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct HouseID;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct House {
    pub id: HouseID,
    pub exterior: PolyLine,
}

impl House {
    pub fn try_new(map: &mut Map, at: Vec2, axis: Vec2) -> Option<HouseID> {
        let exterior = Self::gen_exterior(at, axis);

        if exterior
            .iter()
            .all(|&p| matches!(map.project(p).kind, ProjectKind::Ground))
        {
            Some(map.houses.insert_with_key(move |id| Self { id, exterior }))
        } else {
            None
        }
    }

    fn gen_exterior(at: Vec2, axis: Vec2) -> PolyLine {
        PolyLine::new(vec![
            at,
            at + axis,
            at + axis + axis.perpendicular(),
            at + axis.perpendicular(),
        ])
    }
}
