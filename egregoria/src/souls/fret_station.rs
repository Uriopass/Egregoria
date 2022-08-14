use crate::map::BuildingID;
use crate::map_dynamic::BuildingInfos;
use crate::vehicles::VehicleID;
use crate::{Egregoria, Selectable, SoulID};
use geom::Transform;

pub struct FreightStation {
    pub building: BuildingID,
    pub active: bool,
    pub trains: Vec<VehicleID>,
}

pub fn freight_station_soul(goria: &mut Egregoria, building: BuildingID) -> Option<SoulID> {
    let map = goria.map();

    let f = FreightStation {
        building,
        active: false,
        trains: vec![],
    };
    let b = map.buildings.get(building)?;

    let height = b.height;
    let obb = b.obb;
    let pos = obb.center();
    let [w2, h2] = obb.axis().map(|x| x.magnitude2());

    drop(map);

    let soul = SoulID(goria.world.spawn((
        f,
        Transform::new(pos.z(height)),
        Selectable {
            radius: w2.max(h2).sqrt() * 0.5,
        },
    )));

    goria.write::<BuildingInfos>().set_owner(building, soul);

    Some(soul)
}
