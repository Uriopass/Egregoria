use crate::{Houses, Lot, Roads, SpatialMap};
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
    pub walkway: Polygon,
}

impl House {
    pub fn make(
        houses: &mut Houses,
        spatial_map: &mut SpatialMap,
        roads: &Roads,
        lot: Lot,
    ) -> Option<HouseID> {
        let at = lot.shape.center();
        let axis = lot.road_edge.vec().perpendicular().normalize();

        let mut exterior = Self::gen_exterior(lot.size);

        exterior.rotate(-axis).translate(at);

        let mut ext = exterior.segment(exterior.0.len() - 1);
        ext.resize(3.0);

        let mut walkway = ext.to_polygon();
        let r = &roads[lot.parent];
        walkway.extrude(
            0,
            r.generated_points.project_dist(ext.src) - r.width * 0.5 + 3.0,
        );

        let bbox = exterior.bbox();
        let id = houses.insert_with_key(move |id| Self {
            id,
            exterior,
            walkway,
        });
        spatial_map.insert(id, bbox);
        Some(id)
    }

    pub fn gen_exterior(size: f32) -> Polygon {
        fn rand_in(min: f32, max: f32) -> f32 {
            min + rand::random::<f32>() * (max - min)
        }

        let a = rand_in(15.0, 20.0);
        let b = rand_in(15.0, 20.0);

        let w = f32::max(a, b) * (size / 40.0);
        let h = f32::min(a, b) * (size / 40.0);

        let mut p = Polygon::rect(w, h);
        let corn_coeff = rand_in(0.5, 0.75);
        let seg = rand_in(0.0, 3.99) as usize;

        p.split_segment(seg, corn_coeff);
        p.extrude(seg, rand_in(5.0, 10.0));

        p.translate(-p.barycenter());
        p
    }
}
