use crate::gui::{InspectedEntity, Tool};
use crate::input::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::Selectable;
use egregoria::Egregoria;
use geom::Transform;
use legion::IntoQuery;
use legion::{Entity, EntityStore};
use std::sync::Mutex;

pub fn selectable(goria: &Egregoria, uiworld: &mut UiWorld) {
    let mut inspected = uiworld.write::<InspectedEntity>();
    let mouse = uiworld.read::<MouseInfo>();
    let kbinfo = uiworld.read::<KeyboardInfo>();
    let tool = uiworld.read::<Tool>();

    if mouse.just_pressed.contains(&MouseButton::Left) && matches!(*tool, Tool::Hand) {
        inspected.dist2 = std::f32::INFINITY;
        let protec = Mutex::new(inspected);

        <(Entity, &Transform, &Selectable)>::query().par_for_each_chunk(goria.world(), |chunk| {
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
        });
        inspected = protec.into_inner().unwrap();
    }

    if let Some(e) = inspected.e {
        if goria.world().entry_ref(e).is_err() {
            inspected.e = None;
        }
    }

    if kbinfo.just_pressed.contains(&KeyCode::Escape) || matches!(*tool, Tool::Bulldozer) {
        inspected.e = None;
    }
}
