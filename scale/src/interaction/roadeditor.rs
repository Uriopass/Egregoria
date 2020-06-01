use crate::interaction::{InspectedEntity, Tool};
use crate::map_model::{IntersectionComponent, Map};
use specs::prelude::*;
use specs::shred::PanicHandler;

pub struct RoadEditorSystem;

pub struct RoadEditorResource {
    inspect_e: Entity,
}

impl RoadEditorResource {
    pub fn new(world: &mut World) -> Self {
        let e = world.create_entity().build();

        Self { inspect_e: e }
    }
}

#[derive(SystemData)]
pub struct RoadEditorData<'a> {
    tool: Read<'a, Tool>,
    map: Write<'a, Map>,
    self_r: Write<'a, RoadEditorResource, PanicHandler>,
    inspected: Write<'a, InspectedEntity>,
    intersections: WriteStorage<'a, IntersectionComponent>,
}

impl<'a> System<'a> for RoadEditorSystem {
    type SystemData = RoadEditorData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let state: &mut RoadEditorResource = &mut data.self_r;

        if !matches!(*data.tool, Tool::RoadEditor) {
            if data.inspected.e == Some(state.inspect_e) {
                data.inspected.e = None;
                data.inspected.dirty = false;
            }
            return;
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
