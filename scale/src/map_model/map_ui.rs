use crate::engine_interaction::{KeyCode, KeyboardInfo, MouseButton, MouseInfo};
use crate::interaction::{Movable, MovedEvent, Selectable, SelectedEntity};
use crate::map_model::{Intersection, IntersectionComponent, LanePatternBuilder, Map};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, LineToRender, MeshRender};
use crate::rendering::Color;
use specs::prelude::*;
use specs::shred::PanicHandler;
use specs::shrev::{EventChannel, ReaderId};
use specs::world::EntitiesRes;

pub struct MapUISystem;

pub struct MapUIState {
    reader: ReaderId<MovedEvent>,
    pub selected_inter: Option<Entity>,
    pub entities: Vec<Entity>,
    pub pattern_builder: LanePatternBuilder,
    pub map_render_dirty: bool,
}

impl MapUIState {
    pub fn new(world: &mut World) -> Self {
        let reader = world
            .write_resource::<EventChannel<MovedEvent>>()
            .register_reader();

        Self {
            reader,
            selected_inter: None,
            entities: vec![],
            pattern_builder: LanePatternBuilder::new(),
            map_render_dirty: true,
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
        state.map_render_dirty = false;
        // Moved events
        for event in data.moved.read(&mut state.reader) {
            if let Some(rnc) = data.intersections.get(event.entity) {
                data.map.move_intersection(rnc.id, event.new_pos);
                state.map_render_dirty = true;
            }
        }

        // Intersection creation
        if data.kbinfo.just_pressed.contains(&KeyCode::I) {
            let id = data.map.add_intersection(data.mouseinfo.unprojected);
            let intersections = &data.intersections;
            if let Some(x) = data.selected.e.and_then(|x| intersections.get(x)) {
                data.map.connect(x.id, id, &state.pattern_builder.build());
                state.map_render_dirty = true;
            }
            let e = make_inter_entity(
                &data.map.intersections()[id],
                &data.lazy,
                &data.entities,
                &data.map,
            );
            println!("{:?}", e);
            data.selected.e = Some(e);
        }

        // Intersection deletion
        if data.kbinfo.just_pressed.contains(&KeyCode::Backspace) {
            if let Some(e) = data.selected.e {
                if let Some(inter) = data.intersections.get(e) {
                    data.map.remove_intersection(inter.id);
                    state.map_render_dirty = true;
                    data.intersections.remove(e);
                    data.entities.delete(e).unwrap();
                }
            }
            state.deactive_connect(&data.entities);
        }

        if let Some(x) = data.selected.e {
            if data.intersections.contains(x) {
                state.on_inter_select(
                    x,
                    &data.mouseinfo,
                    &mut data.map,
                    &data.intersections,
                    &data.lazy,
                    &data.entities,
                );
                if data.selected.dirty {
                    state.on_select_dirty(&data.intersections, x, &mut data.map);
                }
            } else {
                state.deactive_connect(&data.entities);
            }
        } else if let Some(x) = state.selected_inter {
            state.deactive_connect(&data.entities);

            if data.mouseinfo.just_pressed.contains(&MouseButton::Left) {
                // Unselected with click in empty space
                let id = data.map.add_intersection(data.mouseinfo.unprojected);
                state.map_render_dirty = true;
                let intersections = &data.intersections;
                let lol = intersections.get(x).unwrap();
                data.map.connect(lol.id, id, &state.pattern_builder.build());
                let e = make_inter_entity(
                    &data.map.intersections()[id],
                    &data.lazy,
                    &data.entities,
                    &data.map,
                );
                data.selected.e = Some(e);
            }
        }

        if state.selected_inter.is_some() {
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
        self.selected_inter = None;
        self.entities
            .drain(..)
            .for_each(|e| entities.delete(e).unwrap());
    }

    fn on_select_dirty(
        &mut self,
        intersections: &WriteStorage<IntersectionComponent>,
        selected: Entity,
        map: &mut Map,
    ) {
        let selected_interc = intersections.get(selected).unwrap();
        map.set_intersection_radius(selected_interc.id, selected_interc.radius);
        map.set_intersection_turn_policy(selected_interc.id, selected_interc.turn_policy);
        map.set_intersection_light_policy(selected_interc.id, selected_interc.light_policy);
        self.map_render_dirty = true;
    }

    fn on_inter_select(
        &mut self,
        selected: Entity,
        mouse: &MouseInfo,
        map: &mut Map,
        intersections: &WriteStorage<IntersectionComponent>,
        lazy: &LazyUpdate,
        entities: &EntitiesRes,
    ) {
        match self.selected_inter {
            None => {
                let color = Color {
                    a: 0.5,
                    ..Color::BLUE
                };
                self.entities.push(
                    lazy.create_entity(entities)
                        .with(Transform::new(mouse.unprojected))
                        .with(
                            MeshRender::empty(0.9)
                                .add(CircleRender {
                                    radius: 2.0,
                                    color,
                                    ..Default::default()
                                })
                                .add(LineToRender {
                                    to: selected,
                                    color,
                                    thickness: 4.0,
                                })
                                .build(),
                        )
                        .build(),
                );

                self.selected_inter = Some(selected);
            }
            Some(y) => {
                let selected_interc = intersections.get(selected).unwrap();
                // Already selected, connect the two
                let interc2 = intersections.get(y).unwrap();
                if y != selected {
                    if let Some(id) = map.find_road(selected_interc.id, interc2.id) {
                        map.remove_road(id);
                    }
                    map.connect(
                        interc2.id,
                        selected_interc.id,
                        &self.pattern_builder.build(),
                    );

                    self.map_render_dirty = true;

                    self.deactive_connect(&entities);
                }
            }
        }
    }
}

pub fn make_inter_entity<'a>(
    inter: &Intersection,
    lazy: &LazyUpdate,
    entities: &Entities<'a>,
    map: &Map,
) -> Entity {
    lazy.create_entity(entities)
        .with(IntersectionComponent {
            id: inter.id,
            radius: inter.interface_radius,
            turn_policy: inter.turn_policy,
            light_policy: inter.light_policy,
        })
        .with(MeshRender::simple(
            CircleRender {
                radius: 2.0,
                color: Color {
                    a: 0.2,
                    ..Color::BLUE
                },
                ..CircleRender::default()
            },
            0.2,
        ))
        .with(Transform::new(inter.pos))
        .with(Movable)
        .with(Selectable::new(10.0))
        .build()
}
