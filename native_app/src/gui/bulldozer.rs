use super::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::Egregoria;
use imgui_inspect_derive::*;
use map_model::{Map, ProjectKind};

register_resource_noserialize!(BulldozerState);

#[derive(Default, Inspect)]
pub struct BulldozerState {
    hold: bool,
}

#[profiling::function]
pub fn bulldozer(goria: &Egregoria, uiworld: &mut UiWorld) {
    let tool: &Tool = &*uiworld.read::<Tool>();

    if !matches!(*tool, Tool::Bulldozer) {
        return;
    }

    let mouseinfo: &MouseInfo = &*uiworld.read::<MouseInfo>();
    let map: &Map = &*goria.map();
    let draw: &mut ImmediateDraw = &mut *uiworld.write::<ImmediateDraw>();
    let mut commands = uiworld.commands();
    let state: &BulldozerState = &*uiworld.read::<BulldozerState>();

    let cur_proj = unwrap_ret!(map.project(unwrap_ret!(mouseinfo.unprojected), 0.0));

    let col = if matches!(
        cur_proj.kind,
        ProjectKind::Inter(_) | ProjectKind::Road(_) | ProjectKind::Building(_)
    ) {
        common::config().gui_danger
    } else {
        common::config().gui_disabled
    };

    draw.circle(cur_proj.pos, 2.0).color(col);

    if (!state.hold && mouseinfo.just_pressed.contains(&MouseButton::Left))
        || (state.hold && mouseinfo.pressed.contains(&MouseButton::Left))
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
                commands.map_remove_building(id);
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
