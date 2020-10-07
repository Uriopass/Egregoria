use crate::{Buildings, Lot, Roads, SpatialMap};
use geom::{Polygon, Vec2};
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct BuildingID;
}

#[derive(Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    Workplace,
    Supermarket,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingID,
    pub exterior: Polygon,
    pub walkway: Polygon,
    pub door_pos: Vec2,
    pub kind: BuildingKind,
}

impl Building {
    pub fn make(
        buildings: &mut Buildings,
        spatial_map: &mut SpatialMap,
        roads: &Roads,
        lot: Lot,
        kind: BuildingKind,
    ) -> Option<BuildingID> {
        let at = lot.shape.center();
        let axis = lot.road_edge.vec().normalize();

        let (mut exterior, walkway_seg) = match kind {
            BuildingKind::House => Self::gen_exterior_house(lot.size),
            BuildingKind::Workplace => Self::gen_exterior_workplace(lot.size),
            BuildingKind::Supermarket => Self::gen_exterior_supermarket(lot.size),
        };

        exterior.rotate(axis).translate(at);

        let mut ext = exterior.segment(walkway_seg);
        let door_pos = ext.center();
        ext.resize(3.0);

        let mut walkway = ext.to_polygon();
        let r = &roads[lot.parent];
        walkway.extrude(
            0,
            r.generated_points.project_dist(ext.src) - r.width * 0.5 + 3.0,
        );

        let bbox = exterior.bbox();
        let id = buildings.insert_with_key(move |id| Self {
            id,
            exterior,
            walkway,
            kind,
            door_pos,
        });
        spatial_map.insert(id, bbox);
        Some(id)
    }

    pub fn gen_exterior_workplace(size: f32) -> (Polygon, usize) {
        let a = rand_in(15.0, 20.0);
        let b = rand_in(15.0, 20.0);

        let width = f32::max(a, b) * (size / 40.0) * 1.5;
        let height = f32::min(a, b) * (size / 40.0);

        let mut p = Polygon::rect(width, height);
        let corn_coeff = rand_in(0.2, 0.3);

        p.split_segment(0, corn_coeff);
        p.split_segment(1, 1.0 - corn_coeff / (1.0 - corn_coeff));
        let extrude = rand_in(height * 0.3, height * 0.4);
        p.extrude(2, extrude);
        p.extrude(0, extrude);

        p.translate(-p.barycenter());
        (p, 3)
    }

    pub fn gen_exterior_house(size: f32) -> (Polygon, usize) {
        let a = rand_in(15.0, 20.0);
        let b = rand_in(15.0, 20.0);

        let width = f32::max(a, b) * (size / 40.0);
        let height = f32::min(a, b) * (size / 40.0);

        let mut p = Polygon::rect(width, height);
        let corn_coeff = rand_in(0.5, 0.75);
        let seg = rand_in(0.0, 3.99) as usize;

        p.split_segment(seg, corn_coeff);
        p.extrude(seg, rand_in(5.0, 10.0));

        p.translate(-p.barycenter());
        (p, if seg == 0 { 1 } else { 0 })
    }

    pub fn gen_exterior_supermarket(size: f32) -> (Polygon, usize) {
        let mut h = rand_in(25.0, 30.0);
        let mut w = h + rand_in(5.0, 10.0);

        w *= size / 40.0;
        h *= size / 40.0;

        let mut p = Polygon::rect(w, h);

        p.translate(-p.barycenter());
        (p, 0)
    }
}

fn rand_in(min: f32, max: f32) -> f32 {
    min + rand::random::<f32>() * (max - min)
}
