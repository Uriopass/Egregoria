use crate::gui::{ErrorTooltip, InspectedEntity};
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::engine_interaction::WorldCommand;
use egregoria::map::{ProjectFilter, ProjectKind};
use egregoria::souls::goods_company::GoodsCompany;
use egregoria::Egregoria;
use geom::{Polygon, Vec2};
use std::borrow::Cow;

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

    let newpoly: Polygon = zone
        .iter()
        .enumerate()
        .map(move |(i, x)| {
            if i == scpy.i {
                if let Some((unproj, offset)) = unproj.zip(scpy.offset) {
                    return unproj.xy() - offset;
                }
            }
            *x
        })
        .collect();

    let area = newpoly.area();

    let mut invalidmsg = format!("");

    let bid = comp.building;
    const MAX_ZONE_AREA: f32 = 75000.0;
    if area > MAX_ZONE_AREA {
        invalidmsg = format!("Area too big ({} > {MAX_ZONE_AREA})", area);
    } else {
        if let Some(v) = map
            .spatial_map()
            .query(
                &newpoly,
                ProjectFilter::INTER | ProjectFilter::BUILDING | ProjectFilter::ROAD,
            )
            .filter(move |x| x != &ProjectKind::Building(bid))
            .next()
        {
            invalidmsg = format!("Zone intersects with {:?}", v);
        }
    }

    let isvalid = invalidmsg.is_empty();

    let base_col = if !isvalid {
        uiworld.write::<ErrorTooltip>().msg = Some(Cow::Owned(invalidmsg));
        uiworld.write::<ErrorTooltip>().isworld = true;
        common::config().gui_danger
    } else {
        common::config().gui_primary
    };

    for (p1, p2) in newpoly.iter().zip(newpoly.iter().cycle().skip(1)) {
        draw.line(p1.z(1.0), p2.z(1.0), 2.0).color(base_col);
    }

    for (i, &p) in newpoly.iter().enumerate() {
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

        draw.circle(p.z(1.0), 5.0).color(base_col);
    }

    if let Some(offset) = state.offset {
        if inp.act.contains(&InputAction::Select) {
            inspected.dontclear = true;
        }
        if !inp.act.contains(&InputAction::Select) {
            if let Some(unproj) = inp.unprojected {
                if isvalid {
                    let unproj = unproj.xy();
                    let newpos = unproj - offset;

                    commands.push(WorldCommand::MoveZonePoint {
                        building: comp.building,
                        i: state.i,
                        pos: newpos,
                    });
                }
            }
            state.offset = None;
        }
    }
}
