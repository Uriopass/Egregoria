use crate::{LotID, Map};
use geom::polygon::Polygon;
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
    pub fn make(map: &mut Map, id: LotID) -> Option<HouseID> {
        let lot = &map.lots[id];
        let at = lot.shape.center();
        let axis = lot.road_edge.vec().perpendicular().normalize();

        let mut exterior = Self::gen_exterior();

        exterior.rotate(axis);
        exterior.translate(at);

        let id = map.houses.insert_with_key(move |id| Self { id, exterior });
        map.spatial_map.insert_house(&map.houses[id]);
        Some(id)
    }

    pub fn gen_exterior() -> Polygon {
        fn rand_in(min: f32, max: f32) -> f32 {
            min + rand::random::<f32>() * (max - min)
        }

        let a = rand_in(10.0, 20.0);
        let b = rand_in(10.0, 20.0);

        let w = f32::max(a, b);
        let h = f32::min(a, b);

        let mut p = Polygon::rect(w, h);
        let corn_coeff = rand_in(0.5, 0.75);
        let seg = rand_in(0.0, 3.99) as usize;

        p.split_segment(seg, corn_coeff);
        p.extrude(seg, rand_in(5.0, 10.0));

        p.translate(-p.barycenter());
        p
    }
}
