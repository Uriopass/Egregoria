use crate::gui::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use common::{Z_TOOL, Z_TOOL_BG};
use egregoria::Egregoria;
use map_model::{IntersectionID, LightPolicy, TurnPolicy};
use map_model::{Map, ProjectKind};

#[derive(Clone)]
pub struct IntersectionComponent {
    pub id: IntersectionID,
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

register_resource_noserialize!(RoadEditorResource);
#[derive(Default)]
pub struct RoadEditorResource {
    pub inspect: Option<IntersectionComponent>,
    pub dirty: bool,
}

pub fn roadeditor(goria: &Egregoria, uiworld: &mut UiWorld) {
    let tool = uiworld.read::<Tool>();
    let mouseinfo = uiworld.read::<MouseInfo>();
    let mut state = uiworld.write::<RoadEditorResource>();
    let mut imm_draw = uiworld.write::<ImmediateDraw>();
    let map = goria.read::<Map>();
    let commands = &mut *uiworld.commands();

    if !matches!(*tool, Tool::RoadEditor) {
        state.inspect = None;
        return;
    }

    if let Some(id) = state.inspect.as_ref().map(|x| x.id) {
        if let Some(inter) = map.intersections().get(id) {
            imm_draw
                .circle(inter.pos, 10.0)
                .color(common::config().gui_success.a(0.5))
                .z(Z_TOOL);
        } else {
            state.inspect = None;
        }
    }

    let cur_proj = map.project(mouseinfo.unprojected, 0.0);

    let proj_col;
    let proj_pos;
    if let ProjectKind::Inter(_) = cur_proj.kind {
        proj_col = common::config().gui_primary;
        proj_pos = cur_proj.pos;
    } else {
        proj_col = common::config().gui_disabled;
        proj_pos = mouseinfo.unprojected;
    }

    imm_draw.circle(proj_pos, 10.0).color(proj_col).z(Z_TOOL_BG);

    if mouseinfo.just_pressed.contains(&MouseButton::Left) {
        if let ProjectKind::Inter(id) = cur_proj.kind {
            let inter = &map.intersections()[id];
            state.inspect = Some(IntersectionComponent {
                id,
                turn_policy: inter.turn_policy,
                light_policy: inter.light_policy,
            });
            state.dirty = false;
        }
    }

    if state.dirty {
        if let Some(interc) = &state.inspect {
            commands.map_update_intersection_policy(
                interc.id,
                interc.turn_policy,
                interc.light_policy,
            );
        }
    }
}
