use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseInfo};
use crate::interaction::{MovedEvent, SelectedEntity};
use crate::map_model::{make_inter_entity, IntersectionComponent, LanePattern, Map};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{LineToRender, MeshRender};
use crate::rendering::{Color, BLUE};
use specs::prelude::*;
use specs::shred::{DynamicSystemData, PanicHandler};
use specs::shrev::{EventChannel, ReaderId};
use specs::world::EntitiesRes;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ConnectState {
    Inactive,
    First(Entity),
}

impl ConnectState {
    pub fn is_first(self) -> bool {
        match self {
            ConnectState::First(_) => true,
            _ => false,
        }
    }
}

pub struct RoadGraphSynchronize;

pub struct RoadGraphSynchronizeState {
    reader: ReaderId<MovedEvent>,
    pub connect_state: ConnectState,
    pub rgs_ui: Vec<Entity>,
    pub pattern: LanePattern,
}

impl RoadGraphSynchronizeState {
    pub fn new(world: &mut World) -> Self {
        let reader = world
            .write_resource::<EventChannel<MovedEvent>>()
            .register_reader();

        Self {
            reader,
            connect_state: ConnectState::Inactive,
            rgs_ui: vec![],
            pattern: LanePattern::two_way(1),
        }
    }
}

#[derive(SystemData)]
pub struct RGSData<'a> {
    entities: Entities<'a>,
    lazy: Read<'a, LazyUpdate>,
    self_state: Write<'a, RoadGraphSynchronizeState, PanicHandler>,
    map: Write<'a, Map, PanicHandler>,
    selected: Write<'a, SelectedEntity>,
    moved: Read<'a, EventChannel<MovedEvent>>,
    kbinfo: Read<'a, KeyboardInfo>,
    mouseinfo: Read<'a, MouseInfo>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    transforms: WriteStorage<'a, Transform>,
}

impl<'a> System<'a> for RoadGraphSynchronize {
    type SystemData = RGSData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let state = &mut data.self_state;
        // Moved events
        for event in data.moved.read(&mut state.reader) {
            if let Some(rnc) = data.intersections.get(event.entity) {
                data.map.move_intersection(rnc.id, event.new_pos);
            }
        }

        // Intersection creation
        if data.kbinfo.just_pressed.contains(&KeyCode::I) {
            let id = data.map.add_intersection(data.mouseinfo.unprojected);
            let intersections = &data.intersections;
            if let Some(x) = data.selected.0.and_then(|x| intersections.get(x)) {
                data.map.connect(x.id, id, &state.pattern);
            }
            let e = make_inter_entity(
                &data.map.intersections()[id],
                data.mouseinfo.unprojected,
                &data.lazy,
                &data.entities,
            );
            *data.selected = SelectedEntity(Some(e));
        }

        // Intersection deletion
        if data.kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            if let Some(e) = data.selected.0 {
                if let Some(inter) = data.intersections.get(e) {
                    data.map.remove_intersection(inter.id);
                    data.intersections.remove(e);
                    data.entities.delete(e).unwrap();
                }
            }
            state.deactive_connect(&data.entities);
        }

        if data.kbinfo.just_pressed.contains(&KeyCode::Escape) {
            state.deactive_connect(&data.entities);
        }

        if let Some(x) = data.selected.0 {
            if let Some(interc) = data.intersections.get(x) {
                state.on_inter_select(
                    x,
                    interc,
                    &data.mouseinfo,
                    &mut data.map,
                    &data.intersections,
                    &data.lazy,
                    &data.entities,
                );
            } else {
                state.deactive_connect(&data.entities);
            }
        } else if let ConnectState::First(x) = state.connect_state {
            state.deactive_connect(&data.entities);

            let id = data.map.add_intersection(data.mouseinfo.unprojected);
            let intersections = &data.intersections;
            let lol = intersections.get(x).unwrap();
            data.map.connect(lol.id, id, &state.pattern);
            let e = make_inter_entity(
                &data.map.intersections()[id],
                data.mouseinfo.unprojected,
                &data.lazy,
                &data.entities,
            );
            *data.selected = SelectedEntity(Some(e));
        }

        if state.connect_state.is_first() {
            let line = state.rgs_ui[0];
            let mouse_pos = data.mouseinfo.unprojected;
            data.transforms
                .get_mut(line)
                .map(|x| x.set_position(mouse_pos));
        }
    }

    fn setup(&mut self, world: &mut World) {
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world);
        let state = RoadGraphSynchronizeState::new(world);
        world.insert(state);
    }
}

impl RoadGraphSynchronizeState {
    fn deactive_connect(&mut self, entities: &EntitiesRes) {
        self.connect_state = ConnectState::Inactive;
        self.rgs_ui
            .drain(..)
            .for_each(|e| entities.delete(e).unwrap());
    }

    fn on_inter_select(
        &mut self,
        selected: Entity,
        selected_interc: &IntersectionComponent,
        mouse: &MouseInfo,
        map: &mut Map,
        intersections: &WriteStorage<IntersectionComponent>,
        lazy: &LazyUpdate,
        entities: &EntitiesRes,
    ) {
        map.set_intersection_radius(selected_interc.id, selected_interc.radius);
        map.set_intersection_turn_policy(selected_interc.id, selected_interc.policy);

        match self.connect_state {
            ConnectState::Inactive => {
                let color = Color { a: 0.5, ..BLUE };
                self.rgs_ui.push(
                    lazy.create_entity(entities)
                        .with(Transform::new(mouse.unprojected))
                        .with(MeshRender::simple(
                            LineToRender {
                                to: selected,
                                color,
                                thickness: 4.0,
                            },
                            9,
                        ))
                        .build(),
                );

                self.connect_state = ConnectState::First(selected);
            }
            ConnectState::First(y) => {
                // Already selected, connect the two
                let interc2 = intersections.get(y).unwrap();
                if y != selected {
                    if map.find_road(selected_interc.id, interc2.id).is_some() {
                        map.disconnect(selected_interc.id, interc2.id);
                    }
                    map.connect(interc2.id, selected_interc.id, &self.pattern);

                    self.deactive_connect(&entities);
                }
            }
        }
    }
}
