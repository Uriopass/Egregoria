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
    pub fn try_make(map: &mut Map, at: Vec2, axis: Vec2) -> Option<HouseID> {
        let mut exterior = mods::gen_house()?;

        exterior.rotate(axis);
        exterior.translate(at + axis * exterior.bcircle().radius);

        if exterior
            .iter()
            .all(|&p| matches!(map.project(p).kind, ProjectKind::Ground))
        {
            let id = map.houses.insert_with_key(move |id| Self { id, exterior });
            map.spatial_map.insert_house(&map.houses[id]);
            Some(id)
        } else {
            None
        }
    }
}
