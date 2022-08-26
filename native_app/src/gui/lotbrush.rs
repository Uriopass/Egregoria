use super::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::map::{LotKind, ProjectFilter, ProjectKind};
use egregoria::Egregoria;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct LotBrushResource {
    pub(crate) kind: LotKind,
    pub(crate) radius: f32,
}

#[profiling::function]
pub(crate) fn lotbrush(goria: &Egregoria, uiworld: &mut UiWorld) {
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

    let mpos = unwrap_ret!(mouseinfo.unprojected);
    draw.circle(mpos.up(0.8), res.radius).color(col);

    if mouseinfo.pressed.contains(&MouseButton::Left) {
        for v in map
            .spatial_map()
            .query_around(mpos.xy(), res.radius, ProjectFilter::LOT)
        {
            if let ProjectKind::Lot(id) = v {
                commands.map_build_house(id);
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
