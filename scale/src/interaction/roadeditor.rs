use crate::interaction::{InspectedEntity, Tool};
use crate::map_model::{IntersectionComponent, Map};
use specs::prelude::*;
use specs::world::EntitiesRes;

struct RoadEditor;

#[derive(Default)]
struct RoadEditorResource {
    selected: Option<Entity>,
}

#[derive(SystemData)]
struct RoadEditorData<'a> {
    entities: Entities<'a>,
    lazy: Read<'a, LazyUpdate>,
    tool: Read<'a, Tool>,
    map: Write<'a, Map>,
    self_r: Write<'a, RoadEditorResource>,
    inspected: Write<'a, InspectedEntity>,
    intersections: WriteStorage<'a, IntersectionComponent>,
}

impl<'a> System<'a> for RoadEditor {
    type SystemData = RoadEditorData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let state: &mut RoadEditorResource = &mut data.self_r;

        if !matches!(*data.tool, Tool::RoadEditor) {
            state.set_selected(&data.entities, None);
            return;
        }

        if let Some(e) = state.selected {
            if data.inspected.dirty {
                state.on_select_dirty(&data.intersections, e, &mut data.map);
            }
        }
    }
}

impl RoadEditorResource {
    fn set_selected(&mut self, entities: &EntitiesRes, sel: Option<Entity>) {
        if let Some(e) = self.selected.take() {
            let _ = entities.delete(e);
        }
        self.selected = sel;
    }

    fn on_select_dirty(
        &mut self,
        intersections: &WriteStorage<IntersectionComponent>,
        selected: Entity,
        map: &mut Map,
    ) {
        let selected_interc = intersections.get(selected).unwrap();
        map.update_intersection(selected_interc.id, |inter| {
            inter.turn_policy = selected_interc.turn_policy;
            inter.light_policy = selected_interc.light_policy;
        });
    }
}
