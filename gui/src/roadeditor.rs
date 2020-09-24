use crate::{Tool, Z_TOOL};
use common::inspect::InspectedEntity;
use egregoria::engine_interaction::{MouseButton, MouseInfo};
use egregoria::rendering::meshrender_component::{CircleRender, MeshRender};
use egregoria::rendering::Color;
use egregoria::NoSerialize;
use geom::Transform;
use imgui_inspect_derive::*;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{system, EntityStore, IntoQuery};
use legion::{Entity, World};
use map_model::{IntersectionID, LightPolicy, TurnPolicy};
use map_model::{Map, ProjectKind};

#[derive(Clone, Inspect)]
pub struct IntersectionComponent {
    #[inspect(skip = true)]
    pub id: IntersectionID,
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

pub struct RoadEditorResource {
    inspect_e: Option<Entity>,
    project_entity: Entity,
}

impl RoadEditorResource {
    pub fn new(world: &mut World) -> Self {
        Self {
            inspect_e: None,
            project_entity: world.push((
                Transform::zero(),
                MeshRender::simple(
                    CircleRender {
                        radius: 2.0,
                        color: Color::BLUE,
                        ..Default::default()
                    },
                    Z_TOOL,
                ),
                NoSerialize,
            )),
        }
    }
}

#[system]
#[read_component(IntersectionComponent)]
#[write_component(Transform)]
#[write_component(MeshRender)]
pub fn roadeditor(
    #[resource] tool: &Tool,
    #[resource] map: &mut Map,
    #[resource] mouseinfo: &MouseInfo,
    #[resource] state: &mut RoadEditorResource,
    #[resource] inspected: &mut InspectedEntity,
    sw: &mut SubWorld,
    buf: &mut CommandBuffer,
) {
    let mut entry = sw.entry_mut(state.project_entity).unwrap();
    let mr = entry.get_component_mut::<MeshRender>().unwrap(); // Unwrap ok: defined in new

    if !matches!(*tool, Tool::RoadEditor) {
        mr.hide = true;
        if inspected.e == state.inspect_e {
            inspected.e = None;
            inspected.dirty = false;
        }
        if let Some(e) = state.inspect_e {
            buf.remove(e)
        }
        return;
    }

    mr.hide = false;

    let cur_proj = map.project(mouseinfo.unprojected);

    entry
        .get_component_mut::<Transform>()
        .unwrap() // Unwrap ok: defined in new
        .set_position(cur_proj.pos);

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
