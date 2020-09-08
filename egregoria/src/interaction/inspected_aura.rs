use crate::interaction::InspectedEntity;
use crate::rendering::meshrender_component::{MeshRender, StrokeCircleRender};
use crate::rendering::Color;
use geom::Transform;
use legion::world::SubWorld;
use legion::{system, Entity, EntityStore, IntoQuery, World};

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
pub fn inspected_aura(
    #[state] aura: &InspectedAura,
    #[resource] inspected: &mut InspectedEntity,
    sw: &mut SubWorld,
) {
    let mr = <&mut MeshRender>::query().get_mut(sw, aura.aura).unwrap(); // Unwrap ok: defined in new
    mr.hide = true;

    if let Some(sel) = inspected.e {
        if let Some(pos) = sw
            .entry_mut(sel)
            .unwrap()
            .get_component::<Transform>()
            .ok()
            .map(|x| x.position())
        {
            let (mr, trans) = <(&mut MeshRender, &mut Transform)>::query()
                .get_mut(sw, aura.aura)
                .unwrap(); // Unwrap ok: defined in new
            trans.set_position(pos);
            mr.hide = false;
        }
    }
}
