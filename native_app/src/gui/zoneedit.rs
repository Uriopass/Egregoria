use crate::gui::InspectedEntity;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::WorldCommand;
use egregoria::souls::goods_company::GoodsCompany;
use egregoria::Egregoria;
use geom::Vec2;

#[derive(Default)]
pub(crate) struct ZoneEditState {
    offset: Option<Vec2>,
    i: usize,
}

#[profiling::function]
pub(crate) fn zoneedit(goria: &Egregoria, uiworld: &mut UiWorld) {
    let mut inspected = uiworld.write::<InspectedEntity>();
    let mut state = uiworld.write::<ZoneEditState>();

    let Some(e) = inspected.e else { state.offset = None; return; };

    let Some(comp) = goria.comp::<GoodsCompany>(e) else { return; };

    let map = goria.map();
    let Some(b) = map.buildings().get(comp.building) else { return; };

    let Some(ref zone) = b.zone else { return; };

    let mut draw = uiworld.write::<ImmediateDraw>();
    let inp = uiworld.read::<InputMap>();
    let mut commands = uiworld.commands();

    for (p1, p2) in zone.iter().zip(zone.iter().cycle().skip(1)) {
        draw.line(p1.z(1.0), p2.z(1.0), 2.0)
            .color(common::config().gui_primary);
    }

    for (i, p) in zone.iter().enumerate() {
        if let Some(unproj) = inp.unprojected {
            let unproj = unproj.xy();

            if unproj.is_close(*p, 5.0) {
                draw.circle(p.z(1.1), 5.0)
                    .color(common::config().gui_success);

                if inp.just_act.contains(&InputAction::Select) {
                    state.offset = Some(unproj - *p);
                    state.i = i;
                }
                continue;
            }
        }

        draw.circle(p.z(1.0), 5.0)
            .color(common::config().gui_primary);
    }

    if let Some(offset) = state.offset {
        if inp.act.contains(&InputAction::Select) {
            if let Some(unproj) = inp.unprojected {
                let unproj = unproj.xy();
                let newpos = unproj - offset;

                let oldpos = zone[state.i];

                if !newpos.is_close(oldpos, 0.1) {
                    commands.push(WorldCommand::MoveZonePoint {
                        building: comp.building,
                        i: state.i,
                        pos: newpos,
                    });
                }

                inspected.dontclear = true;
            }
        } else {
            state.offset = None;
        }
    }
}
