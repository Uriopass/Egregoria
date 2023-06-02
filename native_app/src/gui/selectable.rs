use crate::gui::{InspectedBuilding, InspectedEntity, Tool};
use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::Selectable;
use egregoria::map::ProjectFilter;
use egregoria::Egregoria;
use geom::Transform;
use std::sync::Mutex;

/// Selectable allows to select entities by clicking on them
#[profiling::function]
pub(crate) fn selectable(goria: &Egregoria, uiworld: &mut UiWorld) {
    let mut inspected = uiworld.write::<InspectedEntity>();
    let mut inspected_b = uiworld.write::<InspectedBuilding>();
    let inp = uiworld.read::<InputMap>();
    let tool = uiworld.read::<Tool>();

    if inp.just_act.contains(&InputAction::Select)
        && matches!(*tool, Tool::Hand)
        && !inspected.dontclear
    {
        let mut inspectcpy = *inspected;
        inspectcpy.dist2 = f32::INFINITY;
        let protec = Mutex::new(inspectcpy);
        let unproj = unwrap_ret!(inp.unprojected);

        goria
            .world()
            .query::<(&Transform, &Selectable)>()
            .iter_batched(16)
            //.par_bridge()
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

    if inp.just_act.contains(&InputAction::Select)
        && matches!(*tool, Tool::Hand)
        && !inspected_b.dontclear
    {
        inspected_b.e = None;
        if inspected.e.is_none() {
            let unproj = unwrap_ret!(inp.unprojected);
            let map = goria.map();
            inspected_b.e = map
                .spatial_map()
                .query(unproj.xy(), ProjectFilter::BUILDING)
                .find_map(|x| x.as_building());
        }
    }
    inspected.dontclear = false;
    inspected_b.dontclear = false;

    if let Some(e) = inspected.e {
        if !goria.world().contains(e) {
            inspected.e = None;
        }
    }

    if let Some(b) = inspected_b.e {
        if !goria.map().buildings().contains_key(b) {
            inspected_b.e = None;
        }
    }

    if inp.just_act.contains(&InputAction::Close) || matches!(*tool, Tool::Bulldozer) {
        inspected.e = None;
        inspected_b.e = None;
    }
}
