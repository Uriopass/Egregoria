use common::inspect::InspectedEntity;
use egregoria::api::Location;
use egregoria::rendering::meshrender_component::{MeshRender, StrokeCircleRender};
use geom::Color;
use geom::Transform;
use legion::world::SubWorld;
use legion::{system, Entity, EntityStore, IntoQuery, World};
use map_model::Map;

pub struct InspectedAura {
    aura: Entity,
}

impl InspectedAura {
    pub fn new(world: &mut World) -> InspectedAura {
        let mut mr = MeshRender::simple(
            StrokeCircleRender {
                offset: [0.0, 0.0].into(),
                color: Color::gray(0.7),
                radius: 3.0,
                thickness: 0.1,
            },
            0.9,
        );
        mr.hide = true;
        InspectedAura {
            aura: world.push((Transform::zero(), mr)),
        }
    }
}

#[system]
#[write_component(Transform)]
#[write_component(MeshRender)]
#[read_component(Location)]
pub fn inspected_aura(
    #[state] aura: &InspectedAura,
    #[resource] inspected: &mut InspectedEntity,
    #[resource] map: &Map,
    sw: &mut SubWorld,
) {
    let mr = <&mut MeshRender>::query().get_mut(sw, aura.aura).unwrap(); // Unwrap ok: defined in new
    mr.hide = true;

    if let Some(sel) = inspected.e {
        let mut pos = sw
            .entry_mut(sel)
            .unwrap()
            .get_component::<Transform>()
            .ok()
            .map(|x| x.position());

        if let Ok(loc) = sw.entry_ref(sel).unwrap().get_component::<Location>() {
            match *loc {
                Location::Outside => {}
                Location::Vehicle(v) => {
                    pos = sw
                        .entry_mut(v.0)
                        .unwrap()
                        .get_component::<Transform>()
                        .ok()
                        .map(|x| x.position())
                }
                Location::Building(b) => pos = map.buildings().get(b).map(|b| b.door_pos),
            }
        }

        if let Some(pos) = pos {
            let (mr, trans) = <(&mut MeshRender, &mut Transform)>::query()
                .get_mut(sw, aura.aura)
                .unwrap(); // Unwrap ok: defined in new
            trans.set_position(pos);
            mr.hide = false;
        }
    }
}
