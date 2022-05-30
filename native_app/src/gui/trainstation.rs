use super::Tool;
use crate::gui::inputmap::{InputAction, InputMap};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use egregoria::Egregoria;
use geom::{Degrees, OBB};
use map_model::{LanePatternBuilder, ProjectFilter};

pub enum TrainToolKind {
    AddTrain,
    Trainstation,
}

pub struct TrainTool {
    pub kind: TrainToolKind,
    rotation: Degrees,
}

#[profiling::function]
pub fn trainstation(goria: &Egregoria, uiworld: &mut UiWorld) {
    let tool = *uiworld.read::<Tool>();
    if !matches!(tool, Tool::Train) {
        return;
    }

    uiworld.write_or_default::<TrainTool>();
    let mut res = uiworld.write::<TrainTool>();
    if !matches!(res.kind, TrainToolKind::Trainstation) {
        return;
    }

    let inp = uiworld.read::<InputMap>();

    let mut draw = uiworld.write::<ImmediateDraw>();
    let map = goria.map();
    let commands = &mut *uiworld.commands();

    let mpos = unwrap_ret!(inp.unprojected);

    let w = LanePatternBuilder::new().rail(true).n_lanes(1).width();

    let obb = OBB::new(mpos.xy(), res.rotation.vec2(), 230.0, w + 15.0);

    let intersects = map
        .spatial_map()
        .query(obb, ProjectFilter::INTER | ProjectFilter::ROAD)
        .next()
        .is_some();

    let mut col = common::config().gui_primary;
    if intersects {
        col = common::config().gui_danger;
    }
    col.a = 0.5;

    draw.obb(obb, mpos.z + 0.8).color(col);

    if inp.act.contains(&InputAction::Rotate) {
        res.rotation += Degrees(inp.wheel * 10.0);
        res.rotation.normalize();
    }

    if inp.act.contains(&InputAction::Select) && !intersects {
        commands.map_build_trainstation(
            mpos - 115.0 * res.rotation.vec2().z(0.0),
            mpos + 115.0 * res.rotation.vec2().z(0.0),
        );
    }
}

impl Default for TrainTool {
    fn default() -> Self {
        Self {
            kind: TrainToolKind::AddTrain,
            rotation: Degrees(0.0),
        }
    }
}
