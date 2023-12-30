use crate::map::procgen::{gen_exterior_farm, gen_exterior_house, ColoredMesh};
use crate::map::{Buildings, Environment, LanePattern, SpatialMap};
use crate::souls::goods_company::GoodsCompanyID;
use common::descriptions::BuildingGen;
use egui_inspect::debug_inspect_impl;
use geom::{Color, Polygon, Vec2, Vec3, OBB};
use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;

new_key_type! {
    pub struct BuildingID;
}

debug_inspect_impl!(BuildingID);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    GoodsCompany(GoodsCompanyID),
    RailFreightStation,
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

    pub fn is_cached_in_bkinds(&self) -> bool {
        matches!(
            self,
            BuildingKind::RailFreightStation | BuildingKind::ExternalTrading
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StraightRoadGen {
    pub from: Vec3,
    pub to: Vec3,
    pub pattern: LanePattern,
}

pub const MAX_ZONE_AREA: f32 = 50000.0; // in m²

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub poly: Polygon,
    pub area: f32,
    #[serde(default = "unit_x")]
    pub filldir: Vec2,
}

fn unit_x() -> Vec2 {
    Vec2::X
}

impl Zone {
    pub fn new(p: Polygon, filldir: Vec2) -> Self {
        Self {
            area: p.area(),
            poly: p,
            filldir,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: BuildingID,
    pub door_pos: Vec3,
    pub kind: BuildingKind,
    pub mesh: ColoredMesh,
    pub obb: OBB,
    pub height: f32,
    pub zone: Option<Zone>,
}

impl Building {
    pub fn make(
        buildings: &mut Buildings,
        spatial_map: &mut SpatialMap,
        env: &Environment,
        obb: OBB,
        kind: BuildingKind,
        gen: BuildingGen,
        zone: Option<Zone>,
    ) -> Option<BuildingID> {
        let at = obb.center().z(env.height(obb.center())?);
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
        let door_pos = door_pos.rotated_by(axis).z0() + at + Vec3::z(0.1);

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
            if let Some(zone) = zone.clone() {
                spatial_map.insert(id, zone.poly);
            } else {
                spatial_map.insert(id, obb);
            }
            Self {
                id,
                mesh,
                kind,
                door_pos,
                obb,
                height: at.z,
                zone,
            }
        }))
    }
}
