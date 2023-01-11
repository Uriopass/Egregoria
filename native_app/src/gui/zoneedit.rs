use crate::gui::InspectedEntity;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::WorldCommand;
use egregoria::souls::goods_company::GoodsCompany;
use egregoria::Egregoria;
use geom::Vec2;

#[derive(Copy, Clone, Default)]
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

    let scpy = *state;
    let unproj = inp.unprojected;

    let zoneiter = zone.iter().enumerate().map(move |(i, x)| {
        if i == scpy.i {
            if let Some((unproj, offset)) = unproj.zip(scpy.offset) {
                return unproj.xy() - offset;
            }
        }
        *x
    });

    for (p1, p2) in zoneiter.clone().zip(zoneiter.clone().cycle().skip(1)) {
        draw.line(p1.z(1.0), p2.z(1.0), 2.0)
            .color(common::config().gui_primary);
    }

    for (i, p) in zoneiter.enumerate() {
        if let Some(unproj) = inp.unprojected {
            let unproj = unproj.xy();

            if unproj.is_close(p, 5.0) {
                if state.i == i {}
                draw.circle(p.z(1.1), 5.0)
                    .color(common::config().gui_success);

                if inp.just_act.contains(&InputAction::Select) {
                    state.offset = Some(unproj - p);
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
            inspected.dontclear = true;
        }
        if !inp.act.contains(&InputAction::Select) {
            if let Some(unproj) = inp.unprojected {
                let unproj = unproj.xy();
                let newpos = unproj - offset;

                commands.push(WorldCommand::MoveZonePoint {
                    building: comp.building,
                    i: state.i,
                    pos: newpos,
                });
            }
            state.offset = None;
        }
    }
}
