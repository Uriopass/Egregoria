use crate::{Buildings, Road, SpatialMap};
use geom::{Color, LinearColor, Polygon, Rect, Vec2, OBB};
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct BuildingID;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    Workplace,
    Supermarket,
    Farm,
}

impl BuildingKind {
    pub fn size(&self) -> f32 {
        match self {
            BuildingKind::Farm => 80.0,
            _ => 30.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingID,
    pub door_pos: Vec2,
    pub kind: BuildingKind,
    pub draw: Vec<(Polygon, LinearColor)>,
}

impl Building {
    pub fn make(
        buildings: &mut Buildings,
        spatial_map: &mut SpatialMap,
        road: &Road,
        obb: OBB,
        kind: BuildingKind,
    ) -> BuildingID {
        let at = obb.center();
        let axis = (obb.corners[1] - obb.corners[0]).normalize();
        let size = obb.corners[0].distance(obb.corners[1]);

        let (mut draw, mut door_pos) = match kind {
            BuildingKind::House => crate::procgen::gen_exterior_house(size),
            BuildingKind::Workplace => crate::procgen::gen_exterior_workplace(size),
            BuildingKind::Supermarket => crate::procgen::gen_exterior_supermarket(size),
            BuildingKind::Farm => crate::procgen::gen_exterior_farm(size),
        };

        assert!(!draw.is_empty());

        for (poly, _) in &mut draw {
            poly.rotate(axis).translate(at);
        }
        door_pos = door_pos.rotated_by(axis) + at;

        let (rpos, _, dir) = road.generated_points.project_segment_dir(door_pos);

        let walkway = Polygon(vec![
            rpos + (door_pos - rpos).normalize() * (road.width * 0.5 + 0.25) + dir * 1.5,
            rpos + (door_pos - rpos).normalize() * (road.width * 0.5 + 0.25) - dir * 1.5,
            door_pos - dir * 1.5,
            door_pos + dir * 1.5,
        ]);

        draw.push((walkway, Color::gray(0.4).into()));

        let id = buildings.insert_with_key(move |id| Self {
            id,
            draw,
            kind,
            door_pos,
        });
        spatial_map.insert(id, buildings[id].bbox());
        id
    }

    pub fn bbox(&self) -> Rect {
        let mut bbox = self.draw.first().unwrap().0.bbox();
        for (poly, _) in self.draw.iter().skip(1) {
            bbox = bbox.union(poly.bbox());
        }
        bbox
    }
}
