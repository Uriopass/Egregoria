use crate::{Buildings, Lot, Roads, SpatialMap};
use geom::{Color, LinearColor, Polygon, Rect, Vec2};
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
    Farm,
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
        roads: &Roads,
        lot: Lot,
        kind: BuildingKind,
    ) -> Option<BuildingID> {
        let at = lot.shape.center();
        let axis = lot.road_edge.vec().normalize();

        let (mut draw, mut door_pos) = match kind {
            BuildingKind::House => crate::procgen::gen_exterior_house(lot.size),
            BuildingKind::Workplace => crate::procgen::gen_exterior_workplace(lot.size),
            BuildingKind::Supermarket => crate::procgen::gen_exterior_supermarket(lot.size),
            BuildingKind::Farm => crate::procgen::gen_exterior_supermarket(lot.size),
        };

        assert!(!draw.is_empty());

        for (poly, _) in &mut draw {
            poly.rotate(axis).translate(at);
        }
        door_pos = door_pos.rotated_by(axis) + at;

        let r = &roads[lot.parent];
        let (rpos, _, dir) = r.generated_points.project_segment_dir(door_pos);

        let walkway = Polygon(vec![
            rpos + (door_pos - rpos).normalize() * (r.width * 0.5 + 0.25) + dir * 1.5,
            rpos + (door_pos - rpos).normalize() * (r.width * 0.5 + 0.25) - dir * 1.5,
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
        Some(id)
    }

    pub fn bbox(&self) -> Rect {
        let mut bbox = self.draw.first().unwrap().0.bbox();
        for (poly, _) in self.draw.iter().skip(1) {
            bbox = bbox.union(poly.bbox());
        }
        bbox
    }
}
