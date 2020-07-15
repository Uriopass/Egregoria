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
    pub bcirc: Circle,
}

fn rand_in(min: f32, max: f32) -> f32 {
    min + rand::random::<f32>() * (max - min)
}

impl House {
    pub fn try_make(map: &mut Map, at: Vec2, axis: Vec2) -> Option<HouseID> {
        let mut exterior = if rand::random() {
            Self::gen_exterior_sq()
        } else {
            Self::gen_exterior_corner()
        };

        if let Some(house) = mods::gen_house() {
            exterior = house;
        }

        exterior.rotate(axis);
        exterior.translate(at + axis * exterior.bcircle().radius);

        let bcirc = exterior.bcircle();

        if true
            || (map.houses.values().all(|h| !h.bcirc.overlaps(&bcirc))
                && exterior
                    .iter()
                    .all(|&p| matches!(map.project(p).kind, ProjectKind::Ground)))
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

    fn gen_exterior_sq() -> Polygon {
        let w = rand_in(12.0, 22.0);
        let h = rand_in(10.0, 20.0);

        let mut p = Polygon::rect(w, h);
        p.translate(-p.barycenter());
        p
    }

    fn gen_exterior_corner() -> Polygon {
        let a = rand_in(10.0, 20.0);
        let b = rand_in(10.0, 20.0);

        let (w, h) = (a.max(b), a.min(b));

        let mut p = Polygon::rect(w, h);

        let corn_coeff = rand_in(0.5, 0.75);

        let seg = rand_in(0.0, 3.99) as usize;
        p.split_segment(seg, corn_coeff);
        p.extrude(seg, rand_in(5.0, 10.0));

        p.translate(-p.barycenter());
        p
    }
}
