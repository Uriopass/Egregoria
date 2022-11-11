use crate::map::BuildingID;
use crate::map_dynamic::BuildingInfos;
use crate::vehicles::VehicleID;
use crate::{Egregoria, Selectable, SoulID};
use geom::Transform;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Inspect)]
pub struct FreightStation {
    pub building: BuildingID,
    pub trains: Vec<VehicleID>,
    pub waiting_cargo: u32,
}

pub fn freight_station_soul(goria: &mut Egregoria, building: BuildingID) -> Option<SoulID> {
    let map = goria.map();

    let f = FreightStation {
        building,
        trains: vec![],
        waiting_cargo: 0,
    };
    let b = map.buildings.get(building)?;

    let height = b.height;
    let obb = b.obb;
    let pos = obb.center();
    let [w2, h2] = obb.axis().map(|x| x.mag2());

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

#[cfg(test)]
mod tests {
    use crate::map_dynamic::BuildingInfos;
    use crate::souls::human::{spawn_human, HumanDecisionKind};
    use crate::tests::TestCtx;
    use crate::{BuildingGen, BuildingKind, FreightStation, HumanDecision, WorldCommand};
    use geom::{vec2, vec3, OBB};

    #[test]
    fn test_deliver_to_freight_station_incrs_station() {
        let mut test = TestCtx::new();

        test.build_roads(&[vec3(0., 0., 0.), vec3(100., 0., 0.)]);
        let house = test.build_house_near(vec2(50.0, 50.0));
        let human = spawn_human(&mut test.g, house).unwrap();

        test.apply(&[WorldCommand::MapBuildSpecialBuilding(
            OBB::new(vec2(50.0, 50.0), vec2(1.0, 0.0), 5.0, 5.0),
            BuildingKind::RailFretStation,
            BuildingGen::NoWalkway {
                door_pos: vec2(50.0, 50.0),
            },
            vec![],
        )]);
        test.tick();

        let station = test
            .g
            .map()
            .buildings()
            .iter()
            .find(|(_, b)| matches!(b.kind, BuildingKind::RailFretStation))
            .unwrap()
            .0;

        test.g.comp_mut::<HumanDecision>(human.0).unwrap().kind =
            HumanDecisionKind::DeliverAtBuilding(station);

        let binfos = test.g.read::<BuildingInfos>();
        let stationsoul = binfos.owner(station).unwrap();
        drop(binfos);

        for _ in 0..100 {
            test.tick();

            if test
                .g
                .comp::<FreightStation>(stationsoul.0)
                .unwrap()
                .waiting_cargo
                == 1
            {
                return;
            }
        }

        assert!(false);
    }
}
