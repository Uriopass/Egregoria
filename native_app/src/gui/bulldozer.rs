use super::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::map::{BuildingKind, Map, ProjectFilter, ProjectKind};
use egregoria::Egregoria;
use imgui_inspect_derive::Inspect;

#[derive(Copy, Clone, Default, Inspect)]
pub(crate) struct BulldozerState {
    hold: bool,
}

#[profiling::function]
pub(crate) fn bulldozer(goria: &Egregoria, uiworld: &mut UiWorld) {
    let tool: &Tool = &*uiworld.read::<Tool>();

    if !matches!(*tool, Tool::Bulldozer) {
        return;
    }

    let mouseinfo: &MouseInfo = &*uiworld.read::<MouseInfo>();
    let map: &Map = &*goria.map();
    let draw: &mut ImmediateDraw = &mut *uiworld.write::<ImmediateDraw>();
    let mut commands = uiworld.commands();
    let state: &BulldozerState = &*uiworld.read::<BulldozerState>();

    let cur_proj = map.project(unwrap_ret!(mouseinfo.unprojected), 0.0, ProjectFilter::ALL);

    let col = if matches!(
        cur_proj.kind,
        ProjectKind::Inter(_) | ProjectKind::Road(_) | ProjectKind::Building(_)
    ) {
        common::config().gui_danger
    } else {
        common::config().gui_disabled
    };

    draw.circle(cur_proj.pos.up(0.5), 2.0).color(col);

    if ((!state.hold && mouseinfo.just_pressed.contains(&MouseButton::Left))
        || (state.hold && mouseinfo.pressed.contains(&MouseButton::Left)))
        && !matches!(cur_proj.kind, ProjectKind::Ground)
    {
        let mut potentially_empty = Vec::new();
        log::info!("bulldozer {:?}", cur_proj);
        match cur_proj.kind {
            ProjectKind::Inter(id) => {
                potentially_empty.extend(map.intersections()[id].undirected_neighbors(map.roads()));
                commands.map_remove_intersection(id)
            }
            ProjectKind::Road(id) => {
                let r = &map.roads()[id];

                potentially_empty.push(r.src);
                potentially_empty.push(r.dst);

                commands.map_remove_road(id);
            }
            ProjectKind::Building(id) => {
                if let Some(b) = map.buildings().get(id) {
                    if !matches!(b.kind, BuildingKind::ExternalTrading) {
                        commands.map_remove_building(id);
                    }
                }
            }
            ProjectKind::Ground | ProjectKind::Lot(_) => {}
        }

        for id in potentially_empty {
            if map.intersections()[id].roads.is_empty() {
                commands.map_remove_intersection(id);
            }
        }
    }
}
