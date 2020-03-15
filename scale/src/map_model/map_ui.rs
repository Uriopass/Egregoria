use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseInfo};
use crate::interaction::{MovedEvent, SelectedEntity};
use crate::map_model::{make_inter_entity, IntersectionComponent, LanePattern, Map};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, LineToRender, MeshRender};
use crate::rendering::{Color, BLUE};
use cgmath::vec2;
use specs::prelude::*;
use specs::shred::PanicHandler;
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

pub struct MapUISystem;

pub struct MapUIState {
    reader: ReaderId<MovedEvent>,
    pub connect_state: ConnectState,
    pub entities: Vec<Entity>,
    pub pattern: LanePattern,
}

impl MapUIState {
    pub fn new(world: &mut World) -> Self {
        let reader = world
            .write_resource::<EventChannel<MovedEvent>>()
            .register_reader();

        Self {
            reader,
            connect_state: ConnectState::Inactive,
            entities: vec![],
            pattern: LanePattern::two_way(1),
        }
    }
}

#[derive(SystemData)]
pub struct MapUIData<'a> {
    entities: Entities<'a>,
    lazy: Read<'a, LazyUpdate>,
    self_state: Write<'a, MapUIState, PanicHandler>,
    map: Write<'a, Map, PanicHandler>,
    selected: Write<'a, SelectedEntity>,
    moved: Read<'a, EventChannel<MovedEvent>>,
    kbinfo: Read<'a, KeyboardInfo>,
    mouseinfo: Read<'a, MouseInfo>,
    intersections: WriteStorage<'a, IntersectionComponent>,
    transforms: WriteStorage<'a, Transform>,
}

impl<'a> System<'a> for MapUISystem {
    type SystemData = MapUIData<'a>;

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
            if data.intersections.contains(x) {
                state.on_inter_select(
                    x,
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
            let line = state.entities[0];
            let mouse_pos = data.mouseinfo.unprojected;
            if let Some(x) = data.transforms.get_mut(line) {
                x.set_position(mouse_pos);
            }
        }
    }
}

impl MapUIState {
    fn deactive_connect(&mut self, entities: &EntitiesRes) {
        self.connect_state = ConnectState::Inactive;
        self.entities
            .drain(..)
            .for_each(|e| entities.delete(e).unwrap());
    }

    fn on_inter_select<'a>(
        &'a mut self,
        selected: Entity,
        mouse: &'a MouseInfo,
        map: &'a mut Map,
        intersections: &'a WriteStorage<IntersectionComponent>,
        lazy: &'a LazyUpdate,
        entities: &'a EntitiesRes,
    ) {
        let selected_interc = intersections.get(selected).unwrap();
        map.set_intersection_radius(selected_interc.id, selected_interc.radius);
        map.set_intersection_turn_policy(selected_interc.id, selected_interc.policy);

        match self.connect_state {
            ConnectState::Inactive => {
                let color = Color { a: 0.5, ..BLUE };
                self.entities.push(
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

                self.entities.push(
                    lazy.create_entity(entities)
                        .with(Transform::new(vec2(0.0, 0.0)))
                        .with(MeshRender::simple(
                            CircleRender {
                                radius: 10.0,
                                color,
                                ..Default::default()
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
