use crate::gui::{InspectedEntity, Tool};
use crate::input::{MouseButton, MouseInfo};
use common::Z_TOOL;
use egregoria::rendering::immediate::ImmediateDraw;
use geom::Color;
use imgui_inspect_derive::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::Entity;
use legion::{system, IntoQuery};
use map_model::{IntersectionID, LightPolicy, TurnPolicy};
use map_model::{Map, ProjectKind};

#[derive(Clone, Inspect)]
pub struct IntersectionComponent {
    #[inspect(skip = true)]
    pub id: IntersectionID,
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

register_resource_noserialize!(RoadEditorResource);
#[derive(Default)]
pub struct RoadEditorResource {
    inspect_e: Option<Entity>,
}

register_system!(roadeditor);
#[system]
#[read_component(IntersectionComponent)]
pub fn roadeditor(
    #[resource] tool: &Tool,
    #[resource] map: &mut Map,
    #[resource] mouseinfo: &MouseInfo,
    #[resource] state: &mut RoadEditorResource,
    #[resource] inspected: &mut InspectedEntity,
    #[resource] imm_draw: &mut ImmediateDraw,
    sw: &SubWorld,
    buf: &mut CommandBuffer,
) {
    if !matches!(*tool, Tool::RoadEditor) {
        if inspected.e == state.inspect_e {
            inspected.e = None;
            inspected.dirty = false;
        }
        if let Some(e) = state.inspect_e {
            buf.remove(e)
        }
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
            state.inspect_e = Some(buf.push((IntersectionComponent {
                id,
                turn_policy: inter.turn_policy,
                light_policy: inter.light_policy,
            },)));
            inspected.e = state.inspect_e;
        }
    }

    if let Some(insp) = state.inspect_e {
        if inspected.e == Some(insp) && inspected.dirty {
            let selected_interc = <&IntersectionComponent>::query().get(sw, insp).unwrap();
            map.update_intersection(selected_interc.id, |inter| {
                inter.turn_policy = selected_interc.turn_policy;
                inter.light_policy = selected_interc.light_policy;
            });
        }
    }
}
