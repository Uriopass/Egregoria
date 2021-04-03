use super::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use common::Z_TOOL;
use egregoria::Egregoria;
use map_model::{LotKind, ProjectKind};
use serde::{Deserialize, Serialize};

register_resource!(LotBrushResource, "lot_brush");
#[derive(Serialize, Deserialize)]
pub struct LotBrushResource {
    pub kind: LotKind,
    pub radius: f32,
}

pub fn lotbrush(goria: &Egregoria, uiworld: &mut UiWorld) {
    let res = uiworld.read::<LotBrushResource>();
    let tool = *uiworld.read::<Tool>();
    let mouseinfo = uiworld.read::<MouseInfo>();
    let mut draw = uiworld.write::<ImmediateDraw>();
    let map = goria.map();
    let commands = &mut *uiworld.commands();

    if !matches!(tool, Tool::LotBrush) {
        return;
    }

    let kind = res.kind;

    let mut col = match kind {
        LotKind::Unassigned => common::config().lot_unassigned_col,
        LotKind::Residential => common::config().lot_residential_col,
    };

    col.a = 0.2;

    let mpos = mouseinfo.unprojected;
    draw.circle(mpos, res.radius).color(col).z(Z_TOOL);

    if mouseinfo.pressed.contains(&MouseButton::Left) {
        let lots = map.lots();
        for v in map.spatial_map().query_around(mpos, res.radius) {
            if let ProjectKind::Lot(id) = v {
                if lots[id].shape.is_close(mpos, res.radius) {
                    commands.map_build_house(id);
                }
            }
        }
    }
}

impl Default for LotBrushResource {
    fn default() -> Self {
        Self {
            kind: LotKind::Residential,
            radius: 25.0,
        }
    }
}
