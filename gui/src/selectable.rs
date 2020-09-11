use crate::Tool;
use egregoria::engine_interaction::{KeyCode, Selectable};
use egregoria::engine_interaction::{KeyboardInfo, MouseButton, MouseInfo};
use egregoria::ParCommandBuffer;
use geom::Transform;
use legion::world::SubWorld;
use legion::Entity;
use legion::{system, EntityStore};
use std::f32;

#[derive(Default, Debug, Clone, Copy)]
pub struct InspectedEntity {
    pub e: Option<Entity>,
    pub dirty: bool, // Modified by inspection
    pub dist2: f32,
}

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

#[system]
#[read_component(())] // fixme: check if alive works
pub fn selectable_cleanup(
    #[resource] inspected: &mut InspectedEntity,
    #[resource] gy: &mut ParCommandBuffer,
    #[resource] kbinfo: &KeyboardInfo,
    #[resource] tool: &Tool,
    sw: &SubWorld,
) {
    if let Some(e) = inspected.e {
        if !sw.entry_ref(e).is_ok() {
            inspected.e = None;
            return;
        }

        inspected.dist2 = std::f32::INFINITY;

        if kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            gy.kill(e); // Unwrap ok: checked is_alive just before
            inspected.e = None;
        }
    }

    if kbinfo.just_pressed.contains(&KeyCode::Escape) || matches!(*tool, Tool::Bulldozer) {
        inspected.e = None;
    }
}
