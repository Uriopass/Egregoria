use crate::procgen::RoofFace;
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
    pub roofs: Option<Vec<RoofFace>>,
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

        let (mut exterior, walkway_seg, roofs) = match kind {
            BuildingKind::House => crate::procgen::gen_exterior_house(lot.size),
            BuildingKind::Workplace => crate::procgen::gen_exterior_workplace(lot.size),
            BuildingKind::Supermarket => crate::procgen::gen_exterior_supermarket(lot.size),
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
            roofs,
        });
        spatial_map.insert(id, bbox);
        Some(id)
    }
}
