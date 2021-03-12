use super::Tool;
use crate::input::{MouseButton, MouseInfo};
use common::Z_TOOL;
use egregoria::rendering::immediate::ImmediateDraw;
use legion::system;
use map_model::{LotKind, Map, ProjectKind};
use serde::{Deserialize, Serialize};

register_resource!(LotBrushResource, "lot_brush");
#[derive(Serialize, Deserialize)]
pub struct LotBrushResource {
    pub kind: LotKind,
    pub radius: f32,
}

impl Default for LotBrushResource {
    fn default() -> Self {
        Self {
            kind: LotKind::Residential,
            radius: 25.0,
        }
    }
}

register_system!(lotbrush);
#[system]
pub fn lotbrush(
    #[resource] res: &LotBrushResource,
    #[resource] tool: &Tool,
    #[resource] mouseinfo: &MouseInfo,
    #[resource] map: &mut Map,
    #[resource] draw: &mut ImmediateDraw,
) {
    if !matches!(tool, Tool::LotBrush) {
        return;
    }
    let kind = res.kind;

    let mut col = match kind {
        LotKind::Unassigned => unreachable!(),
        LotKind::Residential => common::config().lot_residential_col,
        LotKind::Commercial => common::config().lot_commercial_col,
    };

    col.a = 0.2;

    let mpos = mouseinfo.unprojected;
    draw.circle(mpos, res.radius).color(col).z(Z_TOOL);

    if mouseinfo.buttons.contains(&MouseButton::Left) {
        let lots = map.lots();
        let mut hits = vec![];
        for v in map.spatial_map().query_around(mpos, res.radius) {
            if let ProjectKind::Lot(id) = v {
                if lots[id].shape.is_close(mpos, res.radius) {
                    hits.push(id);
                }
            }
        }

        for hit in hits {
            map.set_lot_kind(hit, kind);
        }
    }
}
