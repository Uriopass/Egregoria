use crate::procgen::ColoredMesh;
use crate::{Buildings, Road, SpatialMap};
use geom::{Color, Polygon, Vec2, OBB};
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct BuildingID;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    Company(u32),
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum BuildingGen {
    House,
    Farm,
    CenteredDoor {
        vertical_factor: f32, // 1.0 means that the door is at the bottom, just on the street
    },
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
        gen: BuildingGen,
    ) -> BuildingID {
        let at = obb.center();
        let axis = (obb.corners[1] - obb.corners[0]).normalize();
        let size = obb.corners[0].distance(obb.corners[1]);

        let r = common::rand::rand2(obb.center().x, obb.center().y).to_bits();

        let (mut mesh, mut door_pos) = match gen {
            BuildingGen::House => crate::procgen::gen_exterior_house(size, r as u64),
            BuildingGen::Farm => crate::procgen::gen_exterior_farm(size, r as u64),
            BuildingGen::CenteredDoor { vertical_factor } => {
                (Default::default(), Vec2::y(-vertical_factor * 0.5 * size))
            }
        };

        for (poly, _) in &mut mesh.faces {
            poly.rotate(axis).translate(at);
        }
        door_pos = door_pos.rotated_by(axis) + at;

        let (rpos, _, dir) = road.points.project_segment_dir(door_pos);

        let walkway = Polygon(vec![
            rpos + (door_pos - rpos).normalize() * (road.width * 0.5 + 0.25) + dir * 1.5,
            rpos + (door_pos - rpos).normalize() * (road.width * 0.5 + 0.25) - dir * 1.5,
            door_pos - dir * 1.5,
            door_pos + dir * 1.5,
        ]);

        mesh.faces.push((walkway, Color::gray(0.4).into()));

        buildings.insert_with_key(move |id| {
            spatial_map.insert(id, mesh.bbox());
            Self {
                id,
                mesh,
                kind,
                door_pos,
                obb,
            }
        })
    }
}
