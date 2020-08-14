use crate::{Lot, LotID, Map, ProjectKind};
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
    pub fn make(map: &mut Map, id: LotID) -> HouseID {
        let lot = &map.lots[id];
        let at = lot.shape.center();
        let axis = lot.road_edge.vec().perpendicular();

        let mut exterior = mods::gen_house()?;

        exterior.rotate(axis);
        exterior.translate(at + axis * exterior.bcircle().radius);

        let id = map.houses.insert_with_key(move |id| Self { id, exterior });
        map.spatial_map.insert_house(&map.houses[id]);
        id
    }
}
