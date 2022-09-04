use crate::gui::{InspectedEntity, Tool};
use crate::input::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::Selectable;
use egregoria::Egregoria;
use geom::Transform;
use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelBridge;
use std::sync::Mutex;

#[profiling::function]
pub(crate) fn selectable(goria: &Egregoria, uiworld: &mut UiWorld) {
    let mut inspected = uiworld.write::<InspectedEntity>();
    let mouse = uiworld.read::<MouseInfo>();
    let kbinfo = uiworld.read::<KeyboardInfo>();
    let tool = uiworld.read::<Tool>();

    if mouse.just_pressed.contains(&MouseButton::Left) && matches!(*tool, Tool::Hand) {
        let mut inspectcpy = *inspected;
        inspectcpy.dist2 = f32::INFINITY;
        let protec = Mutex::new(inspectcpy);
        let unproj = unwrap_ret!(mouse.unprojected);

        goria
            .world()
            .query::<(&Transform, &Selectable)>()
            .iter_batched(16)
            .par_bridge()
            .for_each(|chunk| {
                let mut v = f32::INFINITY;
                let mut ent = None;
                for (e, (trans, select)) in chunk {
                    let dist2 = (trans.position.xy() - unproj.xy()).mag2();
                    if dist2 >= select.radius * select.radius || dist2 >= v {
                        continue;
                    }
                    v = dist2;
                    ent = Some(e);
                }
                let mut inspected = protec.lock().unwrap();
                if inspected.dist2 >= v {
                    inspected.e = ent;
                    inspected.dist2 = v;
                }
            });
        *inspected = protec.into_inner().unwrap();
    }

    if let Some(e) = inspected.e {
        if !goria.world().contains(e) {
            inspected.e = None;
        }
    }

    if kbinfo.just_pressed.contains(&KeyCode::Escape) || matches!(*tool, Tool::Bulldozer) {
        inspected.e = None;
    }
}
