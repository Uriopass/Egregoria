use crate::gui::{InspectedEntity, Tool};
use crate::input::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use egregoria::engine_interaction::Selectable;
use egregoria::ParCommandBuffer;
use geom::Transform;
use legion::world::SubWorld;
use legion::{system, EntityStore};
use legion::{Entity, Query};
use std::sync::Mutex;

register_system!(selectable_select);
#[system]
pub fn selectable_select(
    #[resource] inspected: &mut InspectedEntity,
    #[resource] mouse: &MouseInfo,
    #[resource] tool: &Tool,
    world: &mut SubWorld,
    qry: &mut Query<(Entity, &Transform, &Selectable)>,
) {
    if mouse.just_pressed.contains(&MouseButton::Left) && matches!(*tool, Tool::Hand) {
        let protec = Mutex::new(inspected);

        qry.par_for_each_chunk_mut(world, |chunk| {
            let mut v = std::f32::INFINITY;
            let mut ent = None;
            for (e, trans, select) in chunk {
                let dist2 = (trans.position() - mouse.unprojected).magnitude2();
                if dist2 >= select.radius * select.radius || dist2 >= v {
                    continue;
                }
                v = dist2;
                ent = Some(*e);
            }
            let mut inspected = protec.lock().unwrap();
            if inspected.dist2 >= v {
                inspected.e = ent;
                inspected.dist2 = v;
            }
        })
    }
}

register_system!(selectable_cleanup);
#[system]
#[read_component(())]
pub fn selectable_cleanup(
    #[resource] inspected: &mut InspectedEntity,
    #[resource] gy: &mut ParCommandBuffer,
    #[resource] kbinfo: &KeyboardInfo,
    #[resource] tool: &Tool,
    sw: &SubWorld,
) {
    if let Some(e) = inspected.e {
        if sw.entry_ref(e).is_err() {
            inspected.e = None;
            inspected.dist2 = std::f32::INFINITY;
            return;
        }

        inspected.dist2 = std::f32::INFINITY;

        if kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            gy.kill(e);
            inspected.e = None;
            inspected.dist2 = std::f32::INFINITY;
        }
    }

    if kbinfo.just_pressed.contains(&KeyCode::Escape) || matches!(*tool, Tool::Bulldozer) {
        inspected.e = None;
        inspected.dist2 = std::f32::INFINITY;
    }
}
