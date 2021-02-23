use crate::procgen::ColoredMesh;
use crate::{Buildings, Road, SpatialMap};
use geom::{Color, Polygon, Vec2, OBB};
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct BuildingID;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    Workplace,
    Supermarket,
    CerealFarm,
    CerealFactory,
    AnimalFarm,
    VegetableFarm,
    SlaughterHouse,
    MeatFacility,
    Bakery,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingID,
    pub door_pos: Vec2,
    pub kind: BuildingKind,
    pub mesh: ColoredMesh,
    pub obb: OBB,
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

        let (mut mesh, mut door_pos) = match kind {
            BuildingKind::House => crate::procgen::gen_exterior_house(size, None),
            BuildingKind::Workplace => crate::procgen::gen_exterior_workplace(size),
            BuildingKind::Supermarket => crate::procgen::gen_exterior_supermarket(size),
            BuildingKind::CerealFarm => crate::procgen::gen_exterior_farm(size),
            BuildingKind::CerealFactory => (Default::default(), Vec2::y(-size * 0.3)),
            BuildingKind::Bakery => (Default::default(), Vec2::y(-size * 0.5)),
            BuildingKind::AnimalFarm => crate::procgen::gen_exterior_farm(size),
            BuildingKind::VegetableFarm => crate::procgen::gen_exterior_farm(size),
            BuildingKind::SlaughterHouse => (Default::default(), Vec2::y(-size * 0.5)),
            BuildingKind::MeatFacility => (Default::default(), Vec2::y(-size * 0.3)),
        };

        for (poly, _) in &mut mesh.faces {
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

        mesh.faces.push((walkway, Color::gray(0.4).into()));

        let id = buildings.insert_with_key(move |id| Self {
            id,
            mesh,
            kind,
            door_pos,
            obb,
        });
        spatial_map.insert(id, buildings[id].mesh.bbox());
        id
    }
}
