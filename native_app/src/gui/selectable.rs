use crate::gui::{InspectedEntity, Tool};
use egregoria::engine_interaction::{KeyCode, Selectable};
use egregoria::engine_interaction::{KeyboardInfo, MouseButton, MouseInfo};
use egregoria::ParCommandBuffer;
use geom::Transform;
use legion::world::SubWorld;
use legion::Entity;
use legion::{system, EntityStore};

register_system!(selectable_select);
#[system(for_each)]
pub fn selectable_select(
    #[resource] inspected: &mut InspectedEntity,
    #[resource] mouse: &MouseInfo,
    #[resource] tool: &Tool,
    trans: &Transform,
    select: &Selectable,
    e: &Entity,
) {
    if mouse.just_pressed.contains(&MouseButton::Left) && matches!(*tool, Tool::Hand) {
        let dist2 = (trans.position() - mouse.unprojected).magnitude2();
        if dist2 >= select.radius * select.radius
            || (dist2 >= inspected.dist2 && inspected.e.is_some())
        {
            return;
        }
        inspected.e = Some(*e);
        inspected.dist2 = dist2;
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
            return;
        }

        inspected.dist2 = std::f32::INFINITY;

        if kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            gy.kill(e);
            inspected.e = None;
        }
    }

    if kbinfo.just_pressed.contains(&KeyCode::Escape) || matches!(*tool, Tool::Bulldozer) {
        inspected.e = None;
    }
}
