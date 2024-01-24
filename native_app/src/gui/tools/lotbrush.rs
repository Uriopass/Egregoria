use crate::gui::Tool;
use crate::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use serde::{Deserialize, Serialize};
use simulation::map::{LotKind, ProjectFilter, ProjectKind};
use simulation::Simulation;

#[derive(Serialize, Deserialize)]
pub struct LotBrushResource {
    pub kind: LotKind,
    pub radius: f32,
}

/// Lot brush tool
/// Allows to build houses on lots
pub fn lotbrush(sim: &Simulation, uiworld: &mut UiWorld) {
    profiling::scope!("gui::lotbrush");
    let res = uiworld.read::<LotBrushResource>();
    let tool = *uiworld.read::<Tool>();
    let inp = uiworld.read::<InputMap>();
    let mut draw = uiworld.write::<ImmediateDraw>();
    let map = sim.map();
    let commands = &mut *uiworld.commands();

    if !matches!(tool, Tool::LotBrush) {
        return;
    }

    let kind = res.kind;

    let mut col = match kind {
        LotKind::Unassigned => simulation::config().lot_unassigned_col,
        LotKind::Residential => simulation::config().lot_residential_col,
    };

    col.a = 0.2;

    let mpos = unwrap_ret!(inp.unprojected);
    draw.circle(mpos.up(0.8), res.radius).color(col);

    if inp.act.contains(&InputAction::Select) {
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
