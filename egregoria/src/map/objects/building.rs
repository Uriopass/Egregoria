use crate::map::procgen::{gen_exterior_farm, gen_exterior_house, ColoredMesh};
use crate::map::{Buildings, LanePattern, RoadID, SpatialMap, Terrain};
use crate::souls::goods_company::GoodsCompanyID;
use geom::{Color, Vec2, Vec3, OBB};
use imgui_inspect::debug_inspect_impl;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct BuildingID;
}

debug_inspect_impl!(BuildingID);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    GoodsCompany(GoodsCompanyID),
    RailFretStation,
    TrainStation,
    ExternalTrading,
}

impl BuildingKind {
    pub fn as_goods_company(&self) -> Option<GoodsCompanyID> {
        match self {
            BuildingKind::GoodsCompany(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BuildingGen {
    House,
    Farm,
    CenteredDoor {
        vertical_factor: f32, // 1.0 means that the door is at the bottom, just on the street
    },
    NoWalkway {
        door_pos: Vec2,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StraightRoadGen {
    pub from: Vec3,
    pub to: Vec3,
    pub pattern: LanePattern,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingID,
    pub door_pos: Vec3,
    pub kind: BuildingKind,
    pub mesh: ColoredMesh,
    pub obb: OBB,
    pub height: f32,
    pub attachments: Vec<RoadID>,
}

impl Building {
    pub fn make(
        buildings: &mut Buildings,
        spatial_map: &mut SpatialMap,
        terrain: &Terrain,
        obb: OBB,
        kind: BuildingKind,
        gen: BuildingGen,
        attachments: Vec<RoadID>,
    ) -> Option<BuildingID> {
        let at = obb.center().z(terrain.height(obb.center())?);
        let axis = (obb.corners[1] - obb.corners[0]).normalize();
        let size = obb.corners[0].distance(obb.corners[1]);

        let r = common::rand::rand2(obb.center().x, obb.center().y).to_bits();

        let (mut mesh, door_pos) = match gen {
            BuildingGen::House => gen_exterior_house(size, r as u64),
            BuildingGen::Farm => gen_exterior_farm(size, r as u64),
            BuildingGen::CenteredDoor {
                vertical_factor, ..
            } => (Default::default(), Vec2::y(-vertical_factor * 0.5 * size)),
            BuildingGen::NoWalkway { door_pos } => (Default::default(), door_pos),
        };

        for (poly, _) in &mut mesh.faces {
            for v in poly {
                *v = v.rotate_z(axis) + at;
            }
        }
        let door_pos = door_pos.rotated_by(axis).z0() + at;

        if let BuildingGen::House | BuildingGen::Farm | BuildingGen::CenteredDoor { .. } = gen {
            let bot = obb.segments()[0];
            let rpos = bot.project(door_pos.xy()).z(door_pos.z);
            let dir = bot.vec().normalize().z(0.0);

            let walkway = vec![
                rpos + dir * 1.5,
                rpos - dir * 1.5,
                door_pos - dir * 1.5,
                door_pos + dir * 1.5,
            ];

            mesh.faces.push((walkway, Color::gray(0.4).into()));
        }

        Some(buildings.insert_with_key(move |id| {
            spatial_map.insert(id, obb);
            Self {
                id,
                mesh,
                kind,
                door_pos,
                obb,
                height: at.z,
                attachments,
            }
        }))
    }
}
