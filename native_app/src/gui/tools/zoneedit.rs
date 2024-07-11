use crate::gui::{ErrorTooltip, InspectedBuilding, PotentialCommands};
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use geom::{Polygon, Vec2};
use ordered_float::OrderedFloat;
use simulation::map::{ProjectFilter, ProjectKind, Zone, MAX_ZONE_AREA};
use simulation::world_command::WorldCommand;
use simulation::Simulation;
use std::borrow::Cow;

#[derive(Debug, Copy, Clone, Default)]
pub struct ZoneEditState {
    offset: Option<Vec2>,
    i: usize,
    insert: bool,
}

/// ZoneEdit tool
/// Allows to edit the zone of a building like a farm field or solarpanel field
pub fn zoneedit(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::zoneedit");
    let mut inspected_b = uiworld.write::<InspectedBuilding>();
    let mut state = uiworld.write::<ZoneEditState>();
    let mut potentialcommand = uiworld.write::<PotentialCommands>();

    let Some(bid) = inspected_b.e else {
        state.offset = None;
        return;
    };

    let map = sim.map();
    let Some(b) = map.buildings().get(bid) else {
        return;
    };

    let Some(ref zone) = b.zone else {
        return;
    };
    let filldir = zone.filldir;
    let zone = &zone.poly;

    let mut draw = uiworld.write::<ImmediateDraw>();
    let inp = uiworld.read::<InputMap>();
    let mut commands = uiworld.commands();

    let mut newpoly = Vec::with_capacity(zone.len() + 1);

    for (i, &x) in zone.iter().enumerate() {
        if i == state.i {
            if let Some((unproj, offset)) = inp.unprojected.zip(state.offset) {
                if state.insert {
                    newpoly.push(x);
                }
                newpoly.push(unproj.xy() - offset);
                continue;
            }
        }
        newpoly.push(x)
    }
    let mut newpoly = Polygon(newpoly);
    newpoly.simplify_by(0.003);

    let area = newpoly.area();
    let perimeter = newpoly.perimeter();

    let mut invalidmsg = String::new();

    const MAX_PERIMETER: f32 = 3000.0;
    if area > MAX_ZONE_AREA {
        invalidmsg = format!("Area too big ({area} > {MAX_ZONE_AREA})");
    } else if perimeter > MAX_PERIMETER {
        invalidmsg = format!("Perimeter too big ({perimeter} > {MAX_PERIMETER})");
    } else if !newpoly.contains(b.obb.center()) {
        invalidmsg = String::from("Zone must be near the building");
    } else if let Some(v) = map
        .spatial_map()
        .query(
            &newpoly,
            ProjectFilter::INTER | ProjectFilter::BUILDING | ProjectFilter::ROAD,
        )
        .find(move |x| x != &ProjectKind::Building(bid))
    {
        invalidmsg = format!("Zone intersects with {v:?}");
    }

    let isvalid = invalidmsg.is_empty();

    let base_col = if !isvalid {
        uiworld.write::<ErrorTooltip>().msg = Some(Cow::Owned(invalidmsg));
        uiworld.write::<ErrorTooltip>().isworld = true;
        simulation::colors().gui_danger
    } else {
        simulation::colors().gui_primary
    };

    for (p1, p2) in newpoly.iter().zip(newpoly.iter().cycle().skip(1)) {
        draw.line(p1.z(1.0), p2.z(1.0), 2.0).color(base_col);
    }

    // Find closest interesting point within 20 meters
    let closest = inp.unprojected.and_then(|unproj| {
        newpoly
            .iter()
            .copied()
            .enumerate()
            .map(|(a, b)| (a, b, false))
            .chain(
                // add the segments between points
                newpoly
                    .segments()
                    .enumerate()
                    .map(|(i, x)| (i, x.center(), true)),
            )
            .min_by_key(|(_, x, _)| OrderedFloat(x.distance2(unproj.xy())))
            .filter(|(_, x, _)| x.is_close(unproj.xy(), 20.0))
    });

    if inp.just_act.contains(&InputAction::Select) {
        if let Some(unproj) = inp.unprojected {
            if let Some((i, closest, insert)) = closest {
                state.insert = insert;
                state.offset = Some(unproj.xy() - closest);
                state.i = i;
            }
        }
    }

    for (i, &p) in newpoly.iter().enumerate() {
        if Some((i, p, false)) == closest {
            draw.circle(p.z(1.1), 6.0)
                .color(simulation::colors().gui_success);
            continue;
        }

        draw.circle(p.z(1.0), 5.0).color(base_col);
    }

    for (i, p) in newpoly.segments().map(|s| s.center()).enumerate() {
        if Some((i, p, true)) == closest {
            draw.circle(p.z(1.1), 3.0)
                .color(simulation::colors().gui_success);
            continue;
        }

        draw.circle(p.z(1.0), 2.5).color(base_col);
    }

    if state.offset.is_some() {
        let cmd = WorldCommand::UpdateZone {
            building: bid,
            zone: Zone::new(newpoly, filldir),
        };
        if inp.act.contains(&InputAction::Select) {
            inspected_b.dontclear = true;
            if isvalid {
                potentialcommand.set(cmd);
            }
        } else {
            if isvalid {
                commands.push(cmd);
            }
            state.offset = None;
        }
    }
}
