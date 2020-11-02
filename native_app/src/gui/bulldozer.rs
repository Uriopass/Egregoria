use super::Tool;
use crate::gui::Z_TOOL;
use egregoria::engine_interaction::{MouseButton, MouseInfo};
use egregoria::rendering::immediate::ImmediateDraw;
use geom::Color;
use legion::system;
use map_model::{Map, ProjectKind};

#[system]
pub fn bulldozer(
    #[resource] tool: &Tool,
    #[resource] mouseinfo: &MouseInfo,
    #[resource] map: &mut Map,
    #[resource] draw: &mut ImmediateDraw,
) {
    if !matches!(*tool, Tool::Bulldozer) {
        return;
    }

    let cur_proj = map.project(mouseinfo.unprojected);

    draw.circle(cur_proj.pos, 2.0).color(Color::RED).z(Z_TOOL);

    if mouseinfo.just_pressed.contains(&MouseButton::Left) {
        let mut potentially_empty = Vec::new();
        log::info!("bulldozer {:?}", cur_proj);
        match cur_proj.kind {
            ProjectKind::Inter(id) => {
                potentially_empty.extend(map.intersections()[id].neighbors(map.roads()));
                map.remove_intersection(id)
            }
            ProjectKind::Road(id) => {
                let r = &map.roads()[id];

                potentially_empty.push(r.src);
                potentially_empty.push(r.dst);

                map.remove_road(id);
            }
            ProjectKind::Building(id) => {
                map.remove_building(id);
            }
            ProjectKind::Ground | ProjectKind::Lot(_) => {}
        }

        for id in potentially_empty {
            if map.intersections()[id].roads.is_empty() {
                map.remove_intersection(id);
            }
        }
    }
}
