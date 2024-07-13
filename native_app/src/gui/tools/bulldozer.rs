use crate::gui::specialbuilding::SpecialBuildingResource;
use crate::gui::Tool;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egui_inspect::Inspect;
use simulation::map::{BuildingKind, Map, ProjectFilter, ProjectKind};
use simulation::Simulation;

#[derive(Copy, Clone, Default, Inspect)]
pub struct BulldozerState {
    hold: bool,
}

/// Bulldozer tool
/// Allows to remove roads, intersections and buildings
pub fn bulldozer(sim: &Simulation, uiworld: &UiWorld) {
    profiling::scope!("gui::bulldozer");
    let tool: &Tool = &uiworld.read::<Tool>();

    if !matches!(*tool, Tool::Bulldozer) {
        return;
    }

    let inp: &InputMap = &uiworld.read::<InputMap>();
    let map: &Map = &sim.map();
    let draw: &mut ImmediateDraw = &mut uiworld.write::<ImmediateDraw>();
    let mut commands = uiworld.commands();
    let state: &BulldozerState = &uiworld.read::<BulldozerState>();

    let cur_proj = map.project(unwrap_ret!(inp.unprojected), 0.0, ProjectFilter::ALL);

    let col = if matches!(
        cur_proj.kind,
        ProjectKind::Intersection(_) | ProjectKind::Road(_) | ProjectKind::Building(_)
    ) {
        simulation::colors().gui_danger
    } else {
        simulation::colors().gui_disabled
    };

    draw.circle(cur_proj.pos.up(0.5), 2.0).color(col);

    if ((!state.hold && inp.just_act.contains(&InputAction::Select))
        || (state.hold && inp.act.contains(&InputAction::Select)))
        && !matches!(cur_proj.kind, ProjectKind::Ground)
    {
        uiworld.write::<SpecialBuildingResource>().last_obb = None;

        let mut potentially_empty = Vec::new();
        log::info!("bulldozer {:?}", cur_proj);
        match cur_proj.kind {
            ProjectKind::Intersection(id) => {
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
