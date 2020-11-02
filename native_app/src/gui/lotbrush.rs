use super::Tool;
use crate::gui::Z_TOOL;
use egregoria::engine_interaction::{MouseButton, MouseInfo};
use egregoria::rendering::immediate::ImmediateDraw;
use legion::system;
use map_model::{LotKind, Map, ProjectKind};

const BRUSH_RADIUS: f32 = 25.0;

#[system]
pub fn lotbrush(
    #[resource] tool: &Tool,
    #[resource] mouseinfo: &MouseInfo,
    #[resource] map: &mut Map,
    #[resource] draw: &mut ImmediateDraw,
) {
    let kind = match *tool {
        Tool::LotBrush(k) => k,
        _ => return,
    };

    let mut col = match kind {
        LotKind::Residential => common::config().lot_residential_col,
        LotKind::Commercial => common::config().lot_commercial_col,
    };

    col.a = 0.2;

    let mpos = mouseinfo.unprojected;
    draw.circle(mpos, BRUSH_RADIUS).color(col).z(Z_TOOL);

    if mouseinfo.buttons.contains(&MouseButton::Left) {
        let lots = map.lots();
        let mut hits = vec![];
        for v in map.spatial_map().query_around(mpos, BRUSH_RADIUS) {
            if let ProjectKind::Lot(id) = v {
                if lots[id].shape.is_close(mpos, BRUSH_RADIUS) {
                    hits.push(id);
                }
            }
        }

        for hit in hits {
            map.set_lot_kind(hit, kind);
        }
    }
}
