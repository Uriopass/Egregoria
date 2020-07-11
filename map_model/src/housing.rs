use crate::{Map, ProjectKind};
use geom::polygon::Polygon;
use geom::Vec2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct HouseID;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct House {
    pub id: HouseID,
    pub exterior: Polygon,
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

    fn gen_exterior(at: Vec2, axis: Vec2) -> Polygon {
        let size = 12.0;
        Polygon(vec![
            at,
            at + axis * size,
            at + axis * size + axis.perpendicular() * size,
            at + axis.perpendicular() * size,
        ])
    }
}
