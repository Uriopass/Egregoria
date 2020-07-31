use crate::{Map, ProjectKind};
use geom::circle::Circle;
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

        let bcirc = exterior.bcircle();

        if map.houses.values().all(|h| !h.bcirc.overlaps(&bcirc))
            && exterior
                .iter()
                .all(|&p| matches!(map.project(p).kind, ProjectKind::Ground))
        {
            Some(map.houses.insert_with_key(move |id| Self {
                id,
                exterior,
                bcirc,
            }))
        } else {
            None
        }
    }
}
