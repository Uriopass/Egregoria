use crate::engine_interaction::{MouseButton, MouseInfo};
use crate::interaction::{InspectedEntity, Tool, Z_TOOL};
use crate::map_model::{IntersectionComponent, Map, ProjectKind};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::Color;
use specs::prelude::*;
use specs::shred::PanicHandler;

pub struct RoadEditorSystem;

pub struct RoadEditorResource {
    inspect_e: Entity,
    project_entity: Entity,
}

impl RoadEditorResource {
    pub fn new(world: &mut World) -> Self {
        let e = world.create_entity().build();

        Self {
            inspect_e: e,
            project_entity: world
                .create_entity()
                .with(Transform::zero())
                .with(MeshRender::simple(
                    CircleRender {
                        radius: 2.0,
                        color: Color::BLUE,
                        ..Default::default()
                    },
                    Z_TOOL,
                ))
                .build(),
        }
    }
}

#[derive(SystemData)]
pub struct RoadEditorData<'a> {
    tool: Read<'a, Tool>,
    map: Write<'a, Map>,
    mouseinfo: Read<'a, MouseInfo>,
    self_r: Write<'a, RoadEditorResource, PanicHandler>,
    inspected: Write<'a, InspectedEntity>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    meshrender: WriteStorage<'a, MeshRender>,
    trans: WriteStorage<'a, Transform>,
}

impl<'a> System<'a> for RoadEditorSystem {
    type SystemData = RoadEditorData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let state: &mut RoadEditorResource = &mut data.self_r;

        let mr = data.meshrender.get_mut(state.project_entity).unwrap();

        if !matches!(*data.tool, Tool::RoadEditor) {
            mr.hide = true;
            if data.inspected.e == Some(state.inspect_e) {
                data.inspected.e = None;
                data.inspected.dirty = false;
            }
            data.intersections.remove(state.inspect_e);
            return;
        }

        mr.hide = false;

        let cur_proj = data.map.project(data.mouseinfo.unprojected);

        data.trans
            .get_mut(state.project_entity)
            .unwrap()
            .set_position(cur_proj.pos);

        if data.mouseinfo.just_pressed.contains(&MouseButton::Left) {
            if let ProjectKind::Inter(id) = cur_proj.kind {
                let inter = &data.map.intersections()[id];
                data.intersections
                    .insert(
                        state.inspect_e,
                        IntersectionComponent {
                            id,
                            turn_policy: inter.turn_policy,
                            light_policy: inter.light_policy,
                        },
                    )
                    .unwrap();
                data.inspected.e = Some(state.inspect_e);
            }
        }

        if data.inspected.e == Some(state.inspect_e) && data.inspected.dirty {
            let selected_interc = data.intersections.get(state.inspect_e).unwrap();
            data.map.update_intersection(selected_interc.id, |inter| {
                inter.turn_policy = selected_interc.turn_policy;
                inter.light_policy = selected_interc.light_policy;
            });
        }
    }
}
