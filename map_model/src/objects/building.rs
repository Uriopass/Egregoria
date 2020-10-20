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

        let (mut exterior, mut door_pos, mut roofs) = match kind {
            BuildingKind::House => crate::procgen::gen_exterior_house(lot.size),
            BuildingKind::Workplace => crate::procgen::gen_exterior_workplace(lot.size),
            BuildingKind::Supermarket => crate::procgen::gen_exterior_supermarket(lot.size),
        };

        exterior.rotate(axis).translate(at);
        door_pos = door_pos.rotated_by(axis) + at;

        for v in roofs.iter_mut().flatten() {
            v.poly.rotate(axis).translate(at);
            v.normal = v.normal.rotate_z(axis);
        }

        let r = &roads[lot.parent];
        let (rpos, _, dir) = r.generated_points.project_segment_dir(door_pos);

        let walkway = Polygon(vec![
            rpos + dir * 1.5,
            rpos - dir * 1.5,
            door_pos - dir * 1.5,
            door_pos + dir * 1.5,
        ]);

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
