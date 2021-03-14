use crate::gui::Tool;
use crate::input::{MouseButton, MouseInfo};
use crate::rendering::immediate::ImmediateDraw;
use crate::uiworld::UiWorld;
use common::Z_TOOL;
use egregoria::Egregoria;
use geom::Color;
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
    inspect: Option<IntersectionComponent>,
    dirty: bool,
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

    let cur_proj = map.project(mouseinfo.unprojected);
    imm_draw
        .circle(cur_proj.pos, 2.0)
        .color(Color::BLUE)
        .z(Z_TOOL);

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
